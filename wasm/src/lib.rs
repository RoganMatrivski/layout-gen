mod utils;

use eyre::Context;
use wasm_bindgen::prelude::*;

pub fn _get_drawable_rects(xml: &str, width: u32, height: u32) -> eyre::Result<String> {
    let parsed_layout = layout_gen::layout::parse_layout(xml)?;
    let mut tree = taffy::TaffyTree::new();
    let root = parsed_layout.build_taffy_tree(&mut tree)?;
    tree.compute_layout(
        root,
        taffy::Size {
            width: taffy::AvailableSpace::Definite(width as f32),
            height: taffy::AvailableSpace::Definite(height as f32),
        },
    )?;

    let layouts = layout_gen::collect_drawable_rects(&tree, root)?;

    serde_json::to_string_pretty(&layouts).wrap_err("Failed to serialize layouts")
}

#[wasm_bindgen]
pub fn get_drawable_rects(xml: &str, width: u32, height: u32) -> Result<String, JsValue> {
    utils::set_panic_hook();
    _get_drawable_rects(xml, width, height).map_err(|e| JsValue::from_str(&e.to_string()))
}
