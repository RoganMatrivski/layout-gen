use crate::commons::{parse_dimension, parse_length_percentage, parse_length_percentage_auto};
use taffy::prelude::*;

#[derive(Debug, Clone, leaf_derive::FromXmlAttrs)]
pub struct BlockProperties {
    pub size: SizeProp,
    pub min_size: SizeProp,
    pub max_size: SizeProp,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,

    pub flex_grow: f32,
    pub flex_shrink: f32,
}

impl Default for BlockProperties {
    fn default() -> Self {
        Self {
            size: SizeProp {
                width: "auto".into(),
                height: "auto".into(),
            },
            min_size: SizeProp {
                width: "auto".into(),
                height: "auto".into(),
            },
            max_size: SizeProp {
                width: "auto".into(),
                height: "auto".into(),
            },
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
            flex_grow: 1.,
            flex_shrink: 1.,
        }
    }
}
use crate::commons::{EdgeInsets, SizeProp};

impl BlockProperties {
    pub fn to_taffy_style(&self) -> Style {
        // crate::commons
        Style {
            display: Display::Block,
            size: Size {
                width: parse_dimension(&self.size.width),
                height: parse_dimension(&self.size.height),
            },
            min_size: Size {
                width: parse_dimension(&self.min_size.width),
                height: parse_dimension(&self.min_size.height),
            },
            max_size: Size {
                width: parse_dimension(&self.max_size.width),
                height: parse_dimension(&self.max_size.height),
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
