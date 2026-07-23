use crate::commons::{
    EdgeInsets, LeafProperties, parse_dimension, parse_length_percentage,
    parse_length_percentage_auto,
};
use taffy::prelude::*;
#[derive(serde::Serialize, Debug, Clone, leaf_derive::FromXmlAttrs)]
pub struct BlockProperties {
    pub id: Option<String>,
    pub width: String,
    pub height: String,
    pub min_width: String,
    pub min_height: String,
    pub max_width: String,
    pub max_height: String,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,

    pub flex_grow: f32,
    pub flex_shrink: f32,
}

impl Default for BlockProperties {
    fn default() -> Self {
        Self {
            id: None,
            width: "auto".into(),
            height: "auto".into(),
            min_width: "auto".into(),
            min_height: "auto".into(),
            max_width: "auto".into(),
            max_height: "auto".into(),
            padding: EdgeInsets {
                top: "0px".into(),
                right: "0px".into(),
                bottom: "0px".into(),
                left: "0px".into(),
            },
            margin: EdgeInsets {
                top: "0px".into(),
                right: "0px".into(),
                bottom: "0px".into(),
                left: "0px".into(),
            },
            flex_grow: 0.,
            flex_shrink: 1.,
        }
    }
}
impl LeafProperties for BlockProperties {
    fn id(&self) -> Option<String> {
        self.id.clone()
    }

    fn to_taffy_style(&self) -> Style {
        // crate::commons
        Style {
            display: Display::Block,
            size: Size {
                width: parse_dimension(&self.width),
                height: parse_dimension(&self.height),
            },
            min_size: Size {
                width: parse_dimension(&self.min_width),
                height: parse_dimension(&self.min_height),
            },
            max_size: Size {
                width: parse_dimension(&self.max_width),
                height: parse_dimension(&self.max_height),
            },
            padding: Rect {
                top: parse_length_percentage(&self.padding.top),
                right: parse_length_percentage(&self.padding.right),
                bottom: parse_length_percentage(&self.padding.bottom),
                left: parse_length_percentage(&self.padding.left),
            },
            margin: Rect {
                top: parse_length_percentage_auto(&self.margin.top),
                right: parse_length_percentage_auto(&self.margin.right),
                bottom: parse_length_percentage_auto(&self.margin.bottom),
                left: parse_length_percentage_auto(&self.margin.left),
            },
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            ..Default::default()
        }
    }
}
