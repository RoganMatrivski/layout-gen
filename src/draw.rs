use serde::Serialize;
use strum::EnumString;

#[derive(Debug, Default, Clone, EnumString, Serialize)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    #[default]
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Default, Clone, EnumString, Serialize)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Fit {
    #[default]
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

#[derive(Debug, Default, Clone, EnumString, Serialize)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
}

#[derive(Debug, Default, Clone, EnumString, Serialize)]
pub enum Size {
    #[strum(serialize = "sm")]
    #[serde(rename = "sm")]
    Small,
    #[default]
    #[strum(serialize = "md")]
    #[serde(rename = "md")]
    Medium,
    #[strum(serialize = "lg")]
    #[serde(rename = "lg")]
    Large,
    #[strum(serialize = "xl")]
    #[serde(rename = "xl")]
    ExtraLarge,
}

#[derive(Debug, Clone, leaf_derive::FromXmlAttrs, Serialize)]
pub struct DrawProperties {
    pub id: Option<String>,
    pub component: String,
    pub variant: String,
    pub size: Size,

    pub align: Anchor,      // new — where within the rect to place the content
    pub fit: Fit,           // new — how to resolve size-vs-rect mismatch
    pub overflow: Overflow, // new — clip or let content bleed past the rect
    pub opacity: f32,       // new — 0.0–1.0

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<serde_json::Value>,
    pub content_id: Option<u64>,
}

impl Default for DrawProperties {
    fn default() -> Self {
        Self {
            id: None,
            component: String::new(),
            variant: String::new(),
            size: Size::default(),
            align: Anchor::default(),
            fit: Fit::default(),
            overflow: Overflow::default(),
            opacity: 1.0,
            additional_properties: None,
            content_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commons::FromXmlAttrs;
    use roxmltree::Document;

    #[test]
    fn test_parse_additional_property_json() -> eyre::Result<()> {
        let xml =
            r#"<draw component="button" additional-properties='{"theme": "dark", "count": 42}' content-id="99" />"#;
        let doc = Document::parse(xml)?;
        let node = doc.root_element();
        let defaults = DrawProperties::default();
        let draw_props = DrawProperties::from_node(node, &defaults)?;

        assert_eq!(draw_props.component, "button");
        assert_eq!(draw_props.content_id, Some(99));
        let extra = draw_props
            .additional_properties
            .as_ref()
            .expect("should have additional_properties");
        assert_eq!(extra["theme"], "dark");
        assert_eq!(extra["count"], 42);

        let json = serde_json::to_string(&draw_props)?;
        assert!(json.contains(r#""additional_properties":{"count":42,"theme":"dark"}"#));
        assert!(json.contains(r#""content_id":99"#));

        Ok(())
    }

    #[test]
    fn test_parse_additional_property_none() -> eyre::Result<()> {
        let xml = r#"<draw component="button" />"#;
        let doc = Document::parse(xml)?;
        let node = doc.root_element();
        let defaults = DrawProperties::default();
        let draw_props = DrawProperties::from_node(node, &defaults)?;

        assert!(draw_props.additional_properties.is_none());
        assert_eq!(draw_props.content_id, None);

        let json = serde_json::to_string(&draw_props)?;
        assert!(!json.contains("additional_properties"));
        assert!(json.contains(r#""content_id":null"#));

        Ok(())
    }
}
