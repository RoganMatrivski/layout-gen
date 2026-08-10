mod utils;

use layout_gen::RenderRect;
use wasm_bindgen::prelude::*;

// Note, will need LLM to update this if not valid
#[wasm_bindgen(typescript_custom_section)]
const TS_DEFINITIONS: &str = r#"
export type Anchor =
  | "top-left"
  | "top-center"
  | "top-right"
  | "center-left"
  | "center"
  | "center-right"
  | "bottom-left"
  | "bottom-center"
  | "bottom-right";

export type Fit = "fill" | "contain" | "cover" | "none" | "scale-down";

export type Overflow = "visible" | "hidden";

export type Size = "sm" | "md" | "lg" | "xl";

export interface DrawProperties {
  id?: string | null;
  component: string;
  variant: string;
  size: Size;
  align: Anchor;
  fit: Fit;
  overflow: Overflow;
  opacity: number;
  additional_properties?: Record<string, any>;
  content_id?: number | null;
}

export interface RenderRect {
  x: number;
  y: number;
  width: number;
  height: number;
  depth: number;
  label: string;
  draw?: DrawProperties | null;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "RenderRect[]")]
    pub type RenderRectArray;
}

pub fn _get_drawable_rects(xml: &str, width: u32, height: u32) -> eyre::Result<Vec<RenderRect>> {
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

    layout_gen::collect_drawable_rects(&tree, root)
}

#[wasm_bindgen]
pub fn get_drawable_rects(xml: &str, width: u32, height: u32) -> Result<RenderRectArray, JsValue> {
    utils::set_panic_hook();
    let layouts =
        _get_drawable_rects(xml, width, height).map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_wasm_bindgen::to_value(&layouts)
        .map(Into::into)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
