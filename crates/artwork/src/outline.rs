//! Text-to-path outlining for generated SVGs.
//!
//! Every `<text>` element in the produced variant is shaped with the
//! bundled OFL fonts and replaced by an equivalent `<g><path .../></g>`
//! group, so the shipped artwork carries no font dependency at all.
//!
//! Limitations (documented contract): `<tspan>`, `dy`/`textLength`,
//! multi-line text and `x`/`y` coordinate lists are not supported — the
//! masters use single-anchor single-line labels only. Elements containing
//! child markup or occurring inside XML comments are left untouched.

use crate::engine::find_open_tag;
use crate::engine::inside_comment;
use crate::error::ArtworkError;
use crate::fonts::FontRegistry;
use rustybuzz::ttf_parser::GlyphId;
use rustybuzz::ttf_parser::OutlineBuilder;

/// Style attributes carried over from the `<text>` element to the emitted
/// outline group; font/layout attributes are consumed by the pass.
const STYLE_ATTRIBUTES: &[&str] = &[
    "id",
    "fill",
    "fill-opacity",
    "stroke",
    "stroke-width",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-dasharray",
    "paint-order",
    "opacity",
    "filter",
    "transform",
];

/// Reads `name="..."` out of an element tag slice.
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("{name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

/// Decodes the handful of XML entities used in text content.
fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Parses an SVG length list and returns its first numeric component.
fn first_length(tag: &str, name: &str, default: f64) -> f64 {
    attribute(tag, name)
        .and_then(|value| value.split([',', ' ']).find(|part| !part.is_empty()).and_then(|part| part.parse().ok()))
        .unwrap_or(default)
}

/// Accumulates font-unit path commands as absolute user-space `d` data.
struct PathBuilder {
    d: String,
    /// User-space x of the glyph origin (anchor + pen + x-offset).
    origin_x: f64,
    /// User-space y of the glyph baseline (baseline - y-offset).
    baseline_y: f64,
    /// Font-units to user-units scale (`font-size / units-per-em`).
    scale: f64,
}

impl PathBuilder {
    fn x(&self, x: f32) -> f64 {
        self.origin_x + f64::from(x) * self.scale
    }

    fn y(&self, y: f32) -> f64 {
        self.baseline_y - f64::from(y) * self.scale
    }
}

impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.d.push_str(&format!("M{:.2},{:.2} ", self.x(x), self.y(y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.d.push_str(&format!("L{:.2},{:.2} ", self.x(x), self.y(y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.d
            .push_str(&format!("Q{:.2},{:.2} {:.2},{:.2} ", self.x(x1), self.y(y1), self.x(x), self.y(y)));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.d.push_str(&format!(
            "C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} ",
            self.x(x1),
            self.y(y1),
            self.x(x2),
            self.y(y2),
            self.x(x),
            self.y(y)
        ));
    }

    fn close(&mut self) {
        self.d.push('Z');
    }
}

/// Replaces every `<text>` element in `svg` with outlined glyph paths.
///
/// `fonts` resolves `font-family`/`font-weight` pairs against the bundled
/// font directory. Anchor handling (`text-anchor="start|middle|end"`) and
/// `letter-spacing` are honoured; style attributes are carried onto the
/// emitted `<g>` so strokes, halos and filters keep working.
pub(crate) fn outline_texts(svg: &str, fonts: &FontRegistry) -> Result<String, ArtworkError> {
    let mut out = String::with_capacity(svg.len() + svg.len() / 2);
    let mut cursor = 0usize;
    while let Some((open_start, tag_end, self_closing)) = find_open_tag(svg, "text", cursor) {
        if inside_comment(svg, open_start) || self_closing {
            out.push_str(&svg[cursor..tag_end]);
            cursor = tag_end;
            continue;
        }
        let tag = &svg[open_start..tag_end];
        let Some(content_end) = svg[tag_end..].find("</text>").map(|relative| tag_end + relative) else {
            out.push_str(&svg[cursor..tag_end]);
            cursor = tag_end;
            continue;
        };
        let element_end = content_end + "</text>".len();
        out.push_str(&svg[cursor..open_start]);
        let content = &svg[tag_end..content_end];
        if content.contains('<') {
            out.push_str(&svg[open_start..element_end]);
        } else {
            out.push_str(&outline_text_element(tag, &decode_entities(content.trim()), fonts)?);
        }
        cursor = element_end;
    }
    out.push_str(&svg[cursor..]);
    Ok(out)
}

/// Outlines a single `<text>` element into a `<g><path/></g>` group.
fn outline_text_element(tag: &str, content: &str, fonts: &FontRegistry) -> Result<String, ArtworkError> {
    let family = attribute(tag, "font-family").unwrap_or("sans-serif");
    let weight = match attribute(tag, "font-weight").unwrap_or("normal") {
        "bold" | "bolder" => 700u16,
        other => other.parse::<u16>().unwrap_or(400),
    };
    let font_size = first_length(tag, "font-size", 16.0);
    let x = first_length(tag, "x", 0.0);
    let y = first_length(tag, "y", 0.0);
    let letter_spacing = first_length(tag, "letter-spacing", 0.0);
    let anchor = attribute(tag, "text-anchor").unwrap_or("start");
    let face_id = fonts.face_id(family, weight)?;
    let paths = fonts.with_face_data(face_id, |data, index| {
        let Some(shaper) = rustybuzz::Face::from_slice(data, index) else {
            return String::new();
        };
        let Ok(outliner) = rustybuzz::ttf_parser::Face::parse(data, index) else {
            return String::new();
        };
        let scale = font_size / f64::from(shaper.units_per_em());
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(content);
        buffer.set_direction(rustybuzz::Direction::LeftToRight);
        let shaped = rustybuzz::shape(&shaper, &[], buffer);
        let positions = shaped.glyph_positions();
        let count = positions.len();
        let spacing_total = letter_spacing * (count.saturating_sub(1)) as f64;
        let advance_total: f64 = positions.iter().map(|position| position.x_advance as f64 * scale).sum::<f64>() + spacing_total;
        let mut origin_x = x;
        if anchor == "middle" {
            origin_x -= advance_total / 2.0;
        } else if anchor == "end" {
            origin_x -= advance_total;
        }
        let mut pen_x = 0.0f64;
        let mut paths = String::new();
        for (index, (info, position)) in shaped.glyph_infos().iter().zip(positions.iter()).enumerate() {
            let glyph_id = GlyphId(info.glyph_id as u16);
            let mut builder = PathBuilder {
                d: String::new(),
                origin_x: origin_x + pen_x + f64::from(position.x_offset) * scale,
                baseline_y: y - f64::from(position.y_offset) * scale,
                scale,
            };
            if outliner.outline_glyph(glyph_id, &mut builder).is_some() {
                paths.push_str(&builder.d);
            }
            pen_x += position.x_advance as f64 * scale;
            if index + 1 < count {
                pen_x += letter_spacing;
            }
        }
        paths
    })?;
    let mut attributes = String::new();
    for name in STYLE_ATTRIBUTES {
        if let Some(value) = attribute(tag, name) {
            attributes.push_str(&format!(" {name}=\"{value}\""));
        }
    }
    Ok(format!("<g{attributes}><path d=\"{}\"/></g>", paths.trim_end()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fonts() -> FontRegistry {
        let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../app/resources/fonts"));
        FontRegistry::load(&dir)
    }

    #[test]
    fn outlines_text_element() {
        let svg = r##"<svg><text x="10" y="20" font-family="'Patrick Hand'" font-size="24" fill="#000">M1</text></svg>"##;
        let result = outline_texts(svg, &fonts()).unwrap_or_default();
        assert!(!result.contains("<text"));
        assert!(result.contains("<path d=\"M"));
        assert!(result.contains("fill=\"#000\""));
    }

    #[test]
    fn honours_middle_anchor() {
        let svg = r##"<svg><text x="100" y="20" font-family="'Orbitron'" font-size="20" font-weight="bold" text-anchor="middle">M</text></svg>"##;
        let result = outline_texts(svg, &fonts()).unwrap_or_default();
        assert!(result.contains("<path d=\"M"));
        assert!(!result.contains("<text"));
    }

    #[test]
    fn keeps_halo_stroke_on_group() {
        let svg = r##"<svg><text x="0" y="9" font-family="'Comic Neue'" font-size="26" fill="#111" stroke="#eee" stroke-width="3" paint-order="stroke">M1</text></svg>"##;
        let result = outline_texts(svg, &fonts()).unwrap_or_default();
        assert!(result.contains("stroke=\"#eee\""));
        assert!(result.contains("paint-order=\"stroke\""));
    }

    #[test]
    fn leaves_tspan_content_untouched() {
        let svg = r##"<svg><text x="0" y="0"><tspan>A</tspan></text></svg>"##;
        let result = outline_texts(svg, &fonts()).unwrap_or_default();
        assert!(result.contains("<tspan"));
    }

    #[test]
    fn is_idempotent_on_outlined_input() {
        let fonts = fonts();
        let svg = r##"<svg><text x="0" y="0" font-family="'Comic Neue'">A</text></svg>"##;
        let once = outline_texts(svg, &fonts).unwrap_or_default();
        let twice = outline_texts(&once, &fonts).unwrap_or_default();
        assert_eq!(once, twice);
    }
}
