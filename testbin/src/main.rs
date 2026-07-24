use std::io::Read;

use color_eyre::Report;
use layout_gen::{collect_debug_rects, collect_drawable_rects, layout::*};
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
                        Err(e) => {
                            tracing::warn!(?e, "Failed to parse layout");
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
                drawable_only: false,

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
    drawable_only: bool,

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

    fn get_render_rects(&self) -> eyre::Result<Vec<layout_gen::RenderRect>> {
        let Some(layout) = self.layout.clone() else {
            tracing::warn!("Empty layout!");
            return Ok(vec![]);
        };

        let mut tree = taffy::TaffyTree::new();
        let root = layout.build_taffy_tree(&mut tree)?;

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

        let rects = if self.drawable_only {
            collect_drawable_rects(&tree, root)
        } else {
            collect_debug_rects(&tree, root)
        }?;

        Ok(rects)
    }
}

impl eframe::App for PreviewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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

        // Compute once, shared by the tree view, the debug panel, and the canvas.
        let mut rects = self.get_render_rects().expect("Failed to get render rects");

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
                ui.checkbox(&mut self.drawable_only, "Only drawable");

                ui.add_space(8.0);
                if ui.button("Fit to window").clicked() {
                    self.canvas_size = ctx.content_rect().size();
                }

                if ui.button("Copy Debug Layout").clicked() {
                    let dstr = rects
                        .iter()
                        .map(|x| (x.label.clone(), x.x, x.y, x.width, x.height, x.draw.clone()))
                        .map(|(l, x, y, w, h, d)| {
                            format!("{l}\n\tpos:{x}:{y}\n\tsize:{w}x{h}\n\tdrawprops:{d:#?}")
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    ctx.copy_text(dstr);
                }

                ui.add_space(4.0);
                ui.small("Drag the numbers, or click and type.");

                // --- NEW: tree view of the render tree, synced with selection ---
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.heading("Tree");
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .id_salt("tree_view_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for r in &rects {
                            let indent = (r.depth as f32) * 14.0;
                            ui.horizontal(|ui| {
                                ui.add_space(indent);
                                let is_selected = self.selected == Some(r.node_id);
                                let label = format!("{} ({}x{})", r.label, r.width, r.height);
                                if ui.selectable_label(is_selected, label).clicked() {
                                    self.selected = Some(r.node_id);
                                }
                            });
                        }
                    });
            });

        rects.sort_by_key(|r| r.depth);

        // --- debug panel: docked side panel, synced with self.selected ---
        if self.selected.is_some() {
            egui::Panel::right("node_debug")
                .resizable(true)
                .default_size(260.0)
                .show(ui, |ui| {
                    ui.heading("Node Debug");
                    ui.add_space(4.0);

                    if let Some(id) = self.selected {
                        if let Some(r) = rects.iter().find(|r| r.node_id == id) {
                            egui::ScrollArea::vertical()
                                .max_height(400.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.label(format!("NodeId: {:?}", r.node_id));
                                    ui.label(format!("pos: {:.0},{:.0}", r.x, r.y));
                                    ui.label(format!("size: {:.0}x{:.0}", r.width, r.height));
                                    ui.label(format!("depth: {}", r.depth));
                                    if let Some(draw) = &r.draw {
                                        ui.label(format!("{draw:#?}"));
                                    }
                                    ui.label(format!("style prop\n{}", r.style_str));
                                });

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
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

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let (response, painter) =
                        ui.allocate_painter(self.canvas_size, egui::Sense::click());
                    let origin = response.rect.min;

                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let local = pos2(pos.x - origin.x, pos.y - origin.y);
                            self.selected = topmost_rect_at(&rects, local);
                        }
                    }

                    // Which rect (if any) is under the pointer right now.
                    let hovered_id = response
                        .hover_pos()
                        .map(|p| pos2(p.x - origin.x, p.y - origin.y))
                        .and_then(|local| topmost_rect_at(&rects, local));

                    let title_font = egui::FontId::proportional(13.0);
                    let detail_font = egui::FontId::monospace(11.0);

                    // --- pass 1: draw fills, strokes, and a short label-only card ---
                    for r in &rects {
                        let x1 = origin.x + r.x;
                        let y1 = origin.y + r.y;
                        let x2 = x1 + r.width;
                        let y2 = y1 + r.height;
                        let ui_rect = egui::Rect::from_min_max(pos2(x1, y1), pos2(x2, y2));

                        let [red, green, blue] = self.rc.seed(u64::from(r.node_id)).to_rgb_array();
                        let [ir, ig, ib] = [255 - red, 255 - green, 255 - blue];

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

                        // Skip the short label for whichever rect is hovered — the full
                        // card drawn in pass 2 replaces it so we don't double them up.
                        if hovered_id == Some(r.node_id) {
                            continue;
                        }

                        let galley = painter.layout_no_wrap(
                            r.label.clone(),
                            title_font.clone(),
                            egui::Color32::WHITE,
                        );
                        let padding = 4.0;
                        let card_size = galley.size() + egui::vec2(padding * 2.0, padding * 2.0);
                        let card_rect =
                            egui::Rect::from_min_size(pos2(x1 + 4.0, y1 + 4.0), card_size);

                        painter.rect_filled(card_rect, 4.0, egui::Color32::from_black_alpha(150));
                        painter.galley(
                            pos2(card_rect.min.x + padding, card_rect.min.y + padding),
                            galley,
                            egui::Color32::WHITE,
                        );
                    }

                    // --- pass 2: full detail card for the hovered rect only, drawn last so it's on top ---
                    if let Some(hid) = hovered_id {
                        if let Some(r) = rects.iter().find(|r| r.node_id == hid) {
                            let x1 = origin.x + r.x;
                            let y1 = origin.y + r.y;

                            let lines: Vec<(String, egui::FontId, egui::Color32)> = vec![
                                (r.label.clone(), title_font, egui::Color32::WHITE),
                                (
                                    format!(
                                        "{:.0}×{:.0}  @ {:.0},{:.0}",
                                        r.width, r.height, r.x, r.y
                                    ),
                                    detail_font.clone(),
                                    egui::Color32::from_gray(210),
                                ),
                                (
                                    format!("depth {}  id {:?}", r.depth, r.node_id),
                                    detail_font,
                                    egui::Color32::from_gray(170),
                                ),
                            ];

                            let galleys: Vec<_> = lines
                                .into_iter()
                                .map(|(text, font, color)| {
                                    painter.layout_no_wrap(text, font, color)
                                })
                                .collect();

                            let padding = 5.0;
                            let line_gap = 2.0;
                            let card_w = galleys.iter().map(|g| g.size().x).fold(0.0_f32, f32::max)
                                + padding * 2.0;
                            let card_h = galleys.iter().map(|g| g.size().y).sum::<f32>()
                                + line_gap * (galleys.len().saturating_sub(1)) as f32
                                + padding * 2.0;

                            let card_rect = egui::Rect::from_min_size(
                                pos2(x1 + 4.0, y1 + 4.0),
                                egui::vec2(card_w, card_h),
                            );
                            painter.rect_filled(
                                card_rect,
                                4.0,
                                egui::Color32::from_black_alpha(190),
                            );

                            let mut cursor_y = card_rect.min.y + padding;
                            for galley in galleys {
                                let h = galley.size().y;
                                painter.galley(
                                    pos2(card_rect.min.x + padding, cursor_y),
                                    galley,
                                    egui::Color32::WHITE,
                                );
                                cursor_y += h + line_gap;
                            }
                        }
                    }
                });
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
