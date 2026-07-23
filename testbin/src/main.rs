use std::io::Read;

use color_eyre::Report;
use layout_gen::{collect_rects, layout::*};
mod init;

use notify_debouncer_full::{DebounceEventResult, new_debouncer};

// Avoid musl's default allocator due to lackluster performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tracing::instrument]
fn main() -> Result<(), Report> {
    let args = init::initialize()?;
    println!("Hello, world!");

    let layout_path = args.layout_file.unwrap_or_else(|| {
        rfd::FileDialog::new()
            .add_filter("XML File", &["xml"])
            .set_directory(std::env::current_dir().expect("Failed to get working directory"))
            .pick_file()
            .expect("Failed to pick a file")
    });

    let (tx, rx) = flume::bounded::<Layout>(8);

    let random_color = random_color::RandomColor {
        hue: Some(random_color::options::Gamut::Pink),
        luminosity: Some(random_color::options::Luminosity::Light),
        ..Default::default()
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            let load_layout = {
                let tx = tx.clone();
                let ctx = ctx.clone();
                move |path: &std::path::Path| {
                    tracing::debug!(?path, "File changed!");
                    let file = match std::fs::File::open(path) {
                        Ok(f) => f,
                        Err(_) => {
                            tracing::warn!("Failed to open file");
                            return;
                        }
                    };

                    let mut filestr = String::new();
                    if let Err(e) = std::io::BufReader::new(file).read_to_string(&mut filestr) {
                        tracing::warn!("Failed to read file: {e:?}");
                        return;
                    }

                    let layout = match parse_layout(&filestr) {
                        Ok(l) => l,
                        Err(_) => {
                            tracing::warn!("Failed to parse layout");
                            return;
                        }
                    };

                    if let Err(e) = tx.send(layout) {
                        tracing::warn!("Failed to send layout: {e:?}");
                        return;
                    }

                    ctx.request_repaint();
                }
            };

            // fire once manually on startup
            load_layout(&layout_path);

            // then set up the watcher to reuse the same logic
            let load_layout_for_watcher = load_layout.clone(); // see note below
            let mut debouncer = new_debouncer(
                std::time::Duration::from_millis(300),
                None, // tick_rate, None = default
                move |result: DebounceEventResult| match result {
                    Ok(events) => {
                        for event in &events {
                            for path in &event.paths {
                                load_layout_for_watcher(path);
                            }
                        }
                    }
                    Err(errors) => {
                        for e in errors {
                            tracing::warn!("File watcher failed: {e:?}");
                        }
                    }
                },
            )?;

            debouncer.watch(&layout_path, notify::RecursiveMode::NonRecursive)?;

            Ok(Box::new(PreviewerApp {
                layout: None,
                layout_rx: rx,
                rc: random_color,

                _debouncer: std::sync::Arc::new(debouncer),

                // UI state
                selected: None,
                toasts: egui_toast::Toasts::new(),
                canvas_size: egui::vec2(1280.0, 800.0), // sane default, tweak to taste
            }))
        }),
    )?;

    Ok(())
}

use eframe::egui::{self, pos2};
// #[derive(Clone)]
struct PreviewerApp {
    // layout_file: std::path::PathBuf,
    layout: Option<Layout>,
    layout_rx: flume::Receiver<Layout>,
    // render_rects: Vec<layout_gen::RenderRect>,
    rc: random_color::RandomColor,

    _debouncer: std::sync::Arc<
        notify_debouncer_full::Debouncer<
            notify::RecommendedWatcher,
            notify_debouncer_full::RecommendedCache,
        >,
    >,

    // UI Fields
    selected: Option<taffy::NodeId>,

    toasts: egui_toast::Toasts,
    canvas_size: egui::Vec2,
}

impl PreviewerApp {
    // fn new(cc: &eframe::CreationContext<'_>, file: std::path::PathBuf) -> Self {
    //     // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
    //     // Restore app state using cc.storage (requires the "persistence" feature).
    //     // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
    //     // for e.g. egui::PaintCallback.
    //     Self { layout_file: file }
    // }
}

impl eframe::App for PreviewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        while let Ok(layout) = self.layout_rx.try_recv() {
            self.toasts.add(egui_toast::Toast {
                kind: egui_toast::ToastKind::Info,
                text: "Layout file changed!".into(),
                options: egui_toast::ToastOptions::default()
                    .duration_in_seconds(3.0)
                    .show_progress(true)
                    .show_icon(true),
                ..Default::default()
            });
            self.layout = Some(layout)
        }

        self.toasts.show(ui);

        let ctx = ui.ctx().clone();

        // --- NEW: side panel to set painter/canvas size ---
        egui::Panel::right("canvas_controls")
            .resizable(false)
            .default_size(180.0)
            .show(ui, |ui| {
                ui.heading("Canvas");
                ui.add_space(8.0);

                ui.label("Width");
                ui.add(
                    egui::DragValue::new(&mut self.canvas_size.x)
                        .range(50.0..=8000.0)
                        .speed(2.0),
                );

                ui.label("Height");
                ui.add(
                    egui::DragValue::new(&mut self.canvas_size.y)
                        .range(50.0..=8000.0)
                        .speed(2.0),
                );

                ui.add_space(8.0);
                if ui.button("Fit to window").clicked() {
                    self.canvas_size = ctx.content_rect().size();
                }

                if ui.button("Copy Debug Layout").clicked() {
                    let Some(layout) = self.layout.clone() else {
                        tracing::warn!("Empty layout!");
                        return;
                    };

                    let mut tree = taffy::TaffyTree::new();
                    let root = layout.build_taffy_tree(&mut tree).unwrap();

                    // NEW: layout is computed against the user-chosen canvas size,
                    // not the window/viewport size.
                    tree.compute_layout(
                        root,
                        taffy::Size {
                            width: taffy::AvailableSpace::Definite(self.canvas_size.x),
                            height: taffy::AvailableSpace::Definite(self.canvas_size.y),
                        },
                    )
                    .unwrap();

                    let mut rects = collect_rects(&tree, root).unwrap();
                    rects.sort_by_key(|r| r.depth);

                    // Get debug string version of rendered rects
                    let dstr = rects
                        .into_iter()
                        .map(|x| (x.label, x.x, x.y, x.width, x.height))
                        .map(|(l, x, y, w, h)| format!("{l}: {x}:{y} {w}x{h}"))
                        .collect::<Vec<_>>()
                        .join("\n");

                    ctx.copy_text(dstr);
                }

                ui.add_space(4.0);
                ui.small("Drag the numbers, or click and type.");
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(layout) = self.layout.clone() else {
                tracing::warn!("Empty layout!");
                return;
            };

            let mut tree = taffy::TaffyTree::new();
            let root = layout.build_taffy_tree(&mut tree).unwrap();

            // NEW: layout is computed against the user-chosen canvas size,
            // not the window/viewport size.
            tree.compute_layout(
                root,
                taffy::Size {
                    width: taffy::AvailableSpace::Definite(self.canvas_size.x),
                    height: taffy::AvailableSpace::Definite(self.canvas_size.y),
                },
            )
            .unwrap();

            let mut rects = collect_rects(&tree, root).unwrap();
            rects.sort_by_key(|r| r.depth);

            // NEW: whole dashboard scrolls if canvas_size > visible panel size
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let (response, painter) =
                        ui.allocate_painter(self.canvas_size, egui::Sense::click());
                    let origin = response.rect.min;

                    // --- hit-testing: pointer -> canvas-local coords ---
                    let hover_pos = ctx
                        .pointer_hover_pos()
                        .filter(|&pos| !blocked_by_floating_ui(&ctx, pos))
                        .map(|pos| pos2(pos.x - origin.x, pos.y - origin.y));

                    let hovered_id = hover_pos.and_then(|pos| topmost_rect_at(&rects, pos));

                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let local = pos2(pos.x - origin.x, pos.y - origin.y);
                            self.selected = topmost_rect_at(&rects, local);
                        }
                    }

                    // --- draw every rect, offset by canvas origin ---
                    for r in &rects {
                        let x1 = origin.x + r.x;
                        let y1 = origin.y + r.y;
                        let x2 = x1 + r.width;
                        let y2 = y1 + r.height;
                        let ui_rect = egui::Rect::from_min_max(pos2(x1, y1), pos2(x2, y2));

                        let [red, green, blue] = self.rc.seed(u64::from(r.node_id)).to_rgb_array();
                        let [ir, ig, ib] = [255 - red, 255 - green, 255 - blue];
                        // let debugstr = format!("{x1}:{y1}\n{x2}:{y2}");
                        let rectstr = format!("{}: {}x{}", r.label, r.width, r.height);

                        painter.rect_filled(ui_rect, 0, egui::Color32::from_rgb(red, green, blue));

                        let stroke_width = if self.selected == Some(r.node_id) {
                            4.0
                        } else {
                            2.0
                        };
                        painter.rect_stroke(
                            ui_rect,
                            0,
                            (stroke_width, egui::Color32::from_rgb(ir, ig, ib)),
                            egui::StrokeKind::Inside,
                        );

                        painter.text(
                            pos2(x1 + 5.0, y1 + 5.0),
                            egui::Align2::LEFT_TOP,
                            &rectstr,
                            egui::FontId::proportional(16.0),
                            egui::Color32::BLUE,
                        );
                    }

                    // --- tooltip ---
                    if let Some(id) = hovered_id {
                        if let Some(r) = rects.iter().find(|r| r.node_id == id) {
                            if let Some(local) = hover_pos {
                                let screen_pos = pos2(origin.x + local.x, origin.y + local.y);
                                egui::Area::new(egui::Id::new("preview_tooltip"))
                                    .fixed_pos(screen_pos + egui::vec2(12.0, 12.0))
                                    .order(egui::Order::Tooltip)
                                    .show(&ctx, |ui| {
                                        egui::Frame::popup(ui.style()).show(ui, |ui| {
                                            ui.label(format!("{}", r.label));
                                        });
                                    });
                            }
                        }
                    }
                });

            // --- debug panel: unaffected by scrolling, still a floating Window ---
            if let Some(id) = self.selected {
                if let Some(r) = rects.iter().find(|r| r.node_id == id) {
                    egui::Window::new("Node Debug")
                        .default_height(200.0)
                        .show(&ctx, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.label(format!("NodeId: {:?}", r.node_id));
                                    ui.label(format!("pos: {:.0},{:.0}", r.x, r.y));
                                    ui.label(format!("size: {:.0}x{:.0}", r.width, r.height));
                                    ui.label(format!("depth: {}", r.depth));
                                    ui.label(format!(
                                        "style string (ignore constant value it's garbage mem ptr): \n{}",
                                        r.style_str
                                    ));
                                });

                            if ui.button("Close").clicked() {
                                self.selected = None;
                            }

                            if ui.button("Copy Info").clicked() {
                                let info = format!(
                                    "NodeId: {:?}\npos: {:.0},{:.0}\nsize: {:.0}x{:.0}\ndepth: {}\nstyle: {}",
                                    r.node_id, r.x, r.y, r.width, r.height, r.depth, r.style_str
                                );
                                ctx.copy_text(info);
                            }
                        });
                }
            }
        });
    }
}

fn topmost_rect_at(rects: &[layout_gen::RenderRect], pos: egui::Pos2) -> Option<taffy::NodeId> {
    rects
        .iter()
        .filter(|r| {
            egui::Rect::from_min_size(pos2(r.x, r.y), egui::vec2(r.width, r.height)).contains(pos)
        })
        .max_by_key(|r| r.depth)
        .map(|r| r.node_id)
}

fn blocked_by_floating_ui(ctx: &egui::Context, pos: egui::Pos2) -> bool {
    ctx.layer_id_at(pos)
        .map(|layer| layer.order != egui::Order::Background)
        .unwrap_or(false)
}
