use crate::commons::{
    EdgeInsets, LeafProperties, parse_dimension, parse_length_percentage,
    parse_length_percentage_auto,
};
use taffy::prelude::*;

#[derive(Default, Debug, Clone, leaf_derive::FromXmlAttrs)]
pub struct DrawProperties {
    pub id: Option<String>,
    pub component: String,
    pub variant: String,
    pub size: String,
}
