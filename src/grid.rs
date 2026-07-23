use crate::commons::{
    EdgeInsets, SizeProp, parse_dimension, parse_grid_template, parse_length_percentage,
    parse_length_percentage_auto,
};
// use serde::{Deserialize, Serialize};
use taffy::prelude::*;

#[derive(Debug, Clone, leaf_derive::FromXmlAttrs)]
pub struct GridProperties {
    pub columns: String,
    pub rows: String,
    pub gap_row: f32,
    pub gap_column: f32,
    pub size: SizeProp,
    pub min_size: SizeProp,
    pub max_size: SizeProp,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,

    pub flex_grow: f32,
    pub flex_shrink: f32,
}

impl Default for GridProperties {
    fn default() -> Self {
        Self {
            columns: "none".into(),
            rows: "none".into(),
            gap_row: 0.0,
            gap_column: 0.0,
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

impl GridProperties {
    pub fn to_taffy_style(&self) -> Style {
        Style {
            display: Display::Grid,
            grid_template_columns: parse_grid_template(&self.columns),
            grid_template_rows: parse_grid_template(&self.rows),
            gap: Size {
                width: length(self.gap_column),
                height: length(self.gap_row),
            },
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
