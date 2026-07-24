use strum::EnumString;

#[derive(Debug, Default, Clone, EnumString)]
#[strum(serialize_all = "kebab-case")]
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

#[derive(Debug, Default, Clone, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Fit {
    #[default]
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

#[derive(Debug, Default, Clone, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
}

#[derive(Debug, Default, Clone, EnumString)]
pub enum Size {
    #[strum(serialize = "sm")]
    Small,
    #[default]
    #[strum(serialize = "md")]
    Medium,
    #[strum(serialize = "lg")]
    Large,
    #[strum(serialize = "xl")]
    ExtraLarge,
}

#[derive(Debug, Clone, leaf_derive::FromXmlAttrs)]
pub struct DrawProperties {
    pub id: Option<String>,
    pub component: String,
    pub variant: String,
    pub size: Size,

    pub align: Anchor,      // new — where within the rect to place the content
    pub fit: Fit,           // new — how to resolve size-vs-rect mismatch
    pub overflow: Overflow, // new — clip or let content bleed past the rect
    pub opacity: f32,       // new — 0.0–1.0
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
        }
    }
}
