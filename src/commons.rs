use std::str::FromStr;

use taffy::{
    Dimension, GridTemplateComponent, LengthPercentage, LengthPercentageAuto, RepetitionCount,
    style_helpers::{auto, length, repeat},
};

/// Expands 1 or 2 whitespace-separated tokens using CSS-style shorthand:
/// `"10"` -> (10, 10), `"10 20"` -> (10, 20)
fn css_pair_shorthand(s: &str) -> eyre::Result<[String; 2]> {
    let parts = s.split_whitespace().collect::<Vec<_>>();
    let vals = match parts.as_slice() {
        [a] => [*a, *a],
        [a, b] => [*a, *b],
        _ => eyre::bail!("Invalid shorthand string: {s:?}"),
    };
    Ok(vals.map(String::from))
}

/// Expands 1-4 whitespace-separated tokens using CSS box shorthand rules,
/// returning [top, right, bottom, left]
fn css_box_shorthand(s: &str) -> eyre::Result<[String; 4]> {
    let parts = s.split_whitespace().collect::<Vec<_>>();
    let vals = match parts.as_slice() {
        [a] => [*a, *a, *a, *a],
        [v, h] => [*v, *h, *v, *h],
        [t, h, b] => [*t, *h, *b, *h],
        [t, r, b, l] => [*t, *r, *b, *l],
        _ => eyre::bail!("Invalid shorthand string: {s:?}"),
    };
    Ok(vals.map(String::from))
}

// -- parsing helpers: "auto" / "12px" / "50%" -> taffy types --
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct EdgeInsets {
    pub top: String,
    pub right: String,
    pub bottom: String,
    pub left: String,
}

impl FromStr for EdgeInsets {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [top, right, bottom, left] = css_box_shorthand(s)?;
        Ok(Self {
            top,
            right,
            bottom,
            left,
        })
    }
}

// -- parsing helpers: "auto" / "12px" / "50%" -> taffy types --

// taffy 0.11 ships FromStr for Dimension/LengthPercentage/LengthPercentageAuto
// (parses "12px" / "50%" / "auto" directly) — requires the "parse" feature:
//   taffy = { version = "0.11", features = ["parse"] }
// Falls back to a zero/auto default on bad input rather than panicking.

pub fn parse_dimension(s: &str) -> Dimension {
    s.trim().parse().unwrap_or(auto())
}

pub fn parse_length_percentage(s: &str) -> LengthPercentage {
    s.trim().parse().unwrap_or(length(0.0f32))
}

pub fn parse_length_percentage_auto(s: &str) -> LengthPercentageAuto {
    s.trim().parse().unwrap_or(auto())
}

use taffy::{
    MaxTrackSizingFunction, MinTrackSizingFunction, TrackSizingFunction,
    style_helpers::{fr, max_content, min_content, minmax, percent},
};

/// Split a track-list string into top-level tokens, keeping `repeat(...)` /
/// `minmax(...)` calls intact as single tokens.
fn tokenize_tracks(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth -= 1;
                cur.push(c);
            }
            c if c.is_whitespace() && depth == 0 => {
                if !cur.is_empty() {
                    tokens.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

fn parse_min_track(t: &str) -> Option<MinTrackSizingFunction> {
    let t = t.trim();
    match t {
        "auto" => Some(auto()),
        "min-content" => Some(min_content()),
        "max-content" => Some(max_content()),
        _ => {
            if let Some(px) = t.strip_suffix("px") {
                px.trim().parse::<f32>().ok().map(length)
            } else if let Some(pct) = t.strip_suffix('%') {
                pct.trim().parse::<f32>().ok().map(|v| percent(v / 100.0))
            } else {
                t.parse::<f32>().ok().map(length)
            }
        }
    }
}

fn parse_max_track(t: &str) -> Option<MaxTrackSizingFunction> {
    let t = t.trim();
    match t {
        "auto" => Some(auto()),
        "min-content" => Some(min_content()),
        "max-content" => Some(max_content()),
        _ => {
            if let Some(fr_str) = t.strip_suffix("fr") {
                fr_str.trim().parse::<f32>().ok().map(fr)
            } else if let Some(px) = t.strip_suffix("px") {
                px.trim().parse::<f32>().ok().map(length)
            } else if let Some(pct) = t.strip_suffix('%') {
                pct.trim().parse::<f32>().ok().map(|v| percent(v / 100.0))
            } else {
                t.parse::<f32>().ok().map(length)
            }
        }
    }
}

/// Parse a single (non-repeat) track: `auto`, `1fr`, `100px`, `50%`,
/// `min-content`, `max-content`, or `minmax(min, max)`.
/// Returns `TrackSizingFunction` — the *new* 0.11 name for what used to be
/// called `NonRepeatedTrackSizingFunction`.
fn parse_single_track(tok: &str) -> Option<TrackSizingFunction> {
    let t = tok.trim();
    if let Some(inner) = t.strip_prefix("minmax(").and_then(|s| s.strip_suffix(')')) {
        let (min_s, max_s) = inner.split_once(',')?;
        let min = parse_min_track(min_s)?;
        let max = parse_max_track(max_s)?;
        return Some(minmax(min, max));
    }
    match t {
        "auto" => Some(auto()),
        "min-content" => Some(min_content()),
        "max-content" => Some(max_content()),
        _ => {
            if let Some(fr_str) = t.strip_suffix("fr") {
                fr_str.trim().parse::<f32>().ok().map(fr)
            } else if let Some(px) = t.strip_suffix("px") {
                px.trim().parse::<f32>().ok().map(length)
            } else if let Some(pct) = t.strip_suffix('%') {
                pct.trim().parse::<f32>().ok().map(|v| percent(v / 100.0))
            } else {
                t.parse::<f32>().ok().map(length)
            }
        }
    }
}

fn parse_repeat(tok: &str) -> Option<GridTemplateComponent<String>> {
    let inner = tok.strip_prefix("repeat(")?.strip_suffix(')')?;
    let (count_str, rest) = inner.split_once(',')?;
    let count = match count_str.trim() {
        "auto-fill" => RepetitionCount::AutoFill,
        "auto-fit" => RepetitionCount::AutoFit,
        n => RepetitionCount::Count(n.trim().parse().ok()?),
    };
    let tracks: Vec<TrackSizingFunction> = tokenize_tracks(rest)
        .iter()
        .filter_map(|t| parse_single_track(t))
        .collect();
    if tracks.is_empty() {
        return None;
    }
    Some(repeat(count, tracks))
}

/// Parse a full `grid-template-columns`/`rows` value, e.g.
/// `"1fr 2fr auto 100px repeat(3, 1fr) minmax(50px, 1fr)"`.
/// `"none"` / `""` -> empty (implicit/no explicit tracks).
pub fn parse_grid_template(s: &str) -> Vec<GridTemplateComponent<String>> {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("none") {
        return vec![];
    }
    tokenize_tracks(s)
        .into_iter()
        .filter_map(|tok| {
            if tok.starts_with("repeat(") {
                parse_repeat(&tok)
            } else {
                parse_single_track(&tok).map(GridTemplateComponent::Single)
            }
        })
        .collect()
}

pub trait FromXmlAttrs: Sized {
    type Error: std::fmt::Display + Send + Sync + 'static;
    fn from_node(node: roxmltree::Node, defaults: &Self) -> Result<Self, Self::Error>;
}

pub trait LeafProperties {
    fn to_taffy_style(&self) -> taffy::Style;
    fn id(&self) -> Option<String>;
}
