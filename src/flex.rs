use strum::EnumString;
use taffy::prelude::*;

use crate::commons::{
    EdgeInsets, LeafProperties, SizeProp, parse_dimension, parse_length_percentage,
    parse_length_percentage_auto,
};

#[derive(Debug, Default, EnumString, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

#[derive(Debug, Default, EnumString, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    ReverseWrap,
}

#[derive(Debug, Default, EnumString, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum Align {
    #[default]
    Start,
    End,
    Center,
    Stretch,
}

#[derive(Debug, Default, EnumString, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum Justify {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, leaf_derive::FromXmlAttrs)]
pub struct FlexProperties {
    pub id: Option<String>,

    pub direction: FlexDirection,
    pub reverse: bool,
    pub wrap: FlexWrap,
    pub justify_content: Justify,
    pub align_items: Align,
    pub align_content: Align,
    pub gap_row: f32,
    pub gap_column: f32,
    pub grow: f32,
    pub shrink: f32,
    pub basis: String,
    pub align_self: Option<Align>,

    pub size: SizeProp,
    pub min_size: SizeProp,
    pub max_size: SizeProp,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
}

impl Default for FlexProperties {
    fn default() -> Self {
        Self {
            id: None,
            direction: FlexDirection::Row,
            reverse: false,
            wrap: FlexWrap::NoWrap,
            justify_content: Justify::Start,
            align_items: Align::Stretch,
            align_content: Align::Start,
            gap_row: 0.0,
            gap_column: 0.0,
            grow: 0.0,
            shrink: 1.0,
            basis: "auto".to_string(),
            align_self: None,
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
        }
    }
}

impl From<&Align> for AlignItems {
    fn from(a: &Align) -> Self {
        match a {
            Align::Start => AlignItems::START,
            Align::End => AlignItems::END,
            Align::Center => AlignItems::CENTER,
            Align::Stretch => AlignItems::STRETCH,
        }
    }
}

impl From<&Align> for AlignContent {
    fn from(a: &Align) -> Self {
        match a {
            Align::Start => AlignContent::START,
            Align::End => AlignContent::END,
            Align::Center => AlignContent::CENTER,
            Align::Stretch => AlignContent::STRETCH,
        }
    }
}

impl From<&Justify> for JustifyContent {
    fn from(j: &Justify) -> Self {
        match j {
            Justify::Start => JustifyContent::START,
            Justify::End => JustifyContent::END,
            Justify::Center => JustifyContent::CENTER,
            Justify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
            Justify::SpaceAround => JustifyContent::SPACE_AROUND,
            Justify::SpaceEvenly => JustifyContent::SPACE_EVENLY,
        }
    }
}

impl FlexProperties {
    fn resolved_direction(&self) -> taffy::FlexDirection {
        match (&self.direction, self.reverse) {
            (FlexDirection::Row, false) => taffy::FlexDirection::Row,
            (FlexDirection::Row, true) => taffy::FlexDirection::RowReverse,
            (FlexDirection::Column, false) => taffy::FlexDirection::Column,
            (FlexDirection::Column, true) => taffy::FlexDirection::ColumnReverse,
        }
    }

    fn resolved_wrap(&self) -> taffy::FlexWrap {
        match self.wrap {
            FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            FlexWrap::Wrap => taffy::FlexWrap::Wrap,
            FlexWrap::ReverseWrap => taffy::FlexWrap::WrapReverse,
        }
    }
}

impl LeafProperties for FlexProperties {
    fn id(&self) -> Option<String> {
        self.id.clone()
    }

    fn to_taffy_style(&self) -> Style {
        Style {
            display: Display::Flex,
            flex_direction: self.resolved_direction(),
            flex_wrap: self.resolved_wrap(),
            justify_content: Some((&self.justify_content).into()),
            align_items: Some((&self.align_items).into()),
            align_content: Some((&self.align_content).into()),
            align_self: self.align_self.as_ref().map(Into::into),
            gap: Size {
                width: length(self.gap_column),
                height: length(self.gap_row),
            },
            flex_grow: self.grow,
            flex_shrink: self.shrink,
            flex_basis: parse_dimension(&self.basis),
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
            ..Default::default()
        }
    }
}
