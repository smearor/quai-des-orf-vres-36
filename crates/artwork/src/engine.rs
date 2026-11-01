//! String-level substitution engine for parametrized master SVGs.
//!
//! The masters mark their parametrized points with stable element ids
//! (for example `<stop id="tint-mid" ...>`). Substitution is purely
//! textual: the tag carrying the id is located, then the attribute value
//! or the text content inside the tag is replaced.

use crate::error::ArtworkError;

/// Reports whether `position` lies inside an `<!-- ... -->` XML comment.
pub(crate) fn inside_comment(svg: &str, position: usize) -> bool {
    match svg[..position].rfind("<!--") {
        Some(comment_start) => match svg[..position].rfind("-->") {
            Some(comment_end) => comment_start > comment_end,
            None => true,
        },
        None => false,
    }
}

/// Locates the `<...>` tag carrying `id="..."` and returns its byte range.
///
/// Iterates over all occurrences of the id marker and accepts the first one
/// that sits inside a real element tag — occurrences inside XML comments
/// (such as the parametrization documentation in the master header) and
/// inside text content are skipped.
fn element_tag_range(svg: &str, master: &str, id: &str) -> Result<(usize, usize), ArtworkError> {
    let missing = || ArtworkError::MissingElement {
        master: master.to_owned(),
        id: id.to_owned(),
    };
    let id_marker = format!("id=\"{id}\"");
    let mut search_from = 0;
    while let Some(relative) = svg[search_from..].find(&id_marker) {
        let id_position = search_from + relative;
        search_from = id_position + id_marker.len();
        if inside_comment(svg, id_position) {
            continue;
        }
        let Some(tag_start) = svg[..id_position].rfind('<') else {
            continue;
        };
        let is_element_tag = svg[tag_start + 1..].starts_with(|c: char| c.is_ascii_alphabetic())
            && !svg[tag_start..id_position].contains('>')
            && !inside_comment(svg, tag_start);
        if !is_element_tag {
            continue;
        }
        let Some(relative_end) = svg[id_position..].find('>') else {
            continue;
        };
        return Ok((tag_start, id_position + relative_end));
    }
    Err(missing())
}

/// Replaces the value of `attribute` on the element carrying `id`.
pub(crate) fn set_attribute(svg: &str, master: &str, id: &str, attribute: &str, value: &str) -> Result<String, ArtworkError> {
    let (tag_start, tag_end) = element_tag_range(svg, master, id)?;
    let missing = || ArtworkError::MissingAttribute {
        master: master.to_owned(),
        id: id.to_owned(),
        attribute: attribute.to_owned(),
    };
    let attribute_marker = format!("{attribute}=\"");
    let attribute_position = svg[tag_start..tag_end].find(&attribute_marker).ok_or_else(missing)?;
    let value_start = tag_start + attribute_position + attribute_marker.len();
    let value_end = svg[value_start..tag_end].find('"').map(|relative| value_start + relative).ok_or_else(missing)?;
    Ok(format!("{}{value}{}", &svg[..value_start], &svg[value_end..]))
}

/// Extracts the element name from a tag slice (for example `g` for `<g ...>`).
fn element_name(svg: &str, tag_start: usize) -> Option<&str> {
    let name_start = tag_start + 1;
    let name_end = svg[name_start..]
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':'))
        .map(|relative| name_start + relative)?;
    Some(&svg[name_start..name_end])
}

/// Finds the next `<name` opening tag at or after `cursor`, requiring a
/// non-identifier character after the name (so `<g` does not match
/// `<gradient`). Returns the tag start and the position after its `>`,
/// plus whether the tag is self-closing.
pub(crate) fn find_open_tag(svg: &str, name: &str, cursor: usize) -> Option<(usize, usize, bool)> {
    let open_marker = format!("<{name}");
    let mut search_from = cursor;
    while let Some(relative) = svg[search_from..].find(&open_marker) {
        let open_start = search_from + relative;
        let after_name = open_start + open_marker.len();
        let boundary = svg[after_name..].chars().next();
        search_from = after_name;
        if matches!(boundary, Some(c) if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':') {
            continue;
        }
        let tag_end = svg[after_name..].find('>').map(|relative| after_name + relative)?;
        let self_closing = svg[..tag_end].trim_end().ends_with('/');
        return Some((open_start, tag_end + 1, self_closing));
    }
    None
}

/// Replaces the inner markup of the element carrying `id` with `markup`.
///
/// Handles arbitrarily nested content including same-name child elements
/// and self-closing elements. A self-closing target element is rewritten
/// into an open/close pair.
pub(crate) fn set_inner(svg: &str, master: &str, id: &str, markup: &str) -> Result<String, ArtworkError> {
    let missing = || ArtworkError::MissingElement {
        master: master.to_owned(),
        id: id.to_owned(),
    };
    let (tag_start, tag_end) = element_tag_range(svg, master, id)?;
    let name = element_name(svg, tag_start).ok_or_else(missing)?;
    if svg[..tag_end].trim_end().ends_with('/') {
        let open_tag = svg[tag_start..tag_end].trim_end().trim_end_matches('/').trim_end();
        return Ok(format!("{}{open_tag}>{markup}</{name}>{}", &svg[..tag_start], &svg[tag_end + 1..]));
    }
    let close_marker = format!("</{name}>");
    let mut depth = 1usize;
    let mut cursor = tag_end + 1;
    let content_end = loop {
        let next_close = svg[cursor..].find(&close_marker).map(|relative| cursor + relative);
        let Some(close) = next_close else {
            return Err(missing());
        };
        match find_open_tag(svg, name, cursor) {
            Some((open_start, after_tag, self_closing)) if open_start < close => {
                if !self_closing {
                    depth += 1;
                }
                cursor = after_tag;
            }
            _ => {
                depth -= 1;
                if depth == 0 {
                    break close;
                }
                cursor = close + close_marker.len();
            }
        }
    };
    Ok(format!("{}{markup}{}", &svg[..tag_end + 1], &svg[content_end..]))
}

/// Replaces the character data of the `<text>` element carrying `id`.
pub(crate) fn set_text(svg: &str, master: &str, id: &str, content: &str) -> Result<String, ArtworkError> {
    let (tag_start, tag_end) = element_tag_range(svg, master, id)?;
    let missing = || ArtworkError::MissingElement {
        master: master.to_owned(),
        id: id.to_owned(),
    };
    if !svg[tag_start..tag_end].starts_with("<text") {
        return Err(missing());
    }
    let content_start = tag_end + 1;
    let content_end = svg[content_start..]
        .find("</text>")
        .map(|relative| content_start + relative)
        .ok_or_else(missing)?;
    Ok(format!("{}{content}{}", &svg[..content_start], &svg[content_end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MASTER: &str = r##"<svg><stop id="tint-mid" offset="70%" stop-color="#2563EB" /><text id="rank-letter">I</text></svg>"##;

    #[test]
    fn set_attribute_replaces_value() {
        let result = set_attribute(MASTER, "master.svg", "tint-mid", "stop-color", "#DC2626").unwrap_or_default();
        assert_eq!(
            result,
            r##"<svg><stop id="tint-mid" offset="70%" stop-color="#DC2626" /><text id="rank-letter">I</text></svg>"##
        );
    }

    #[test]
    fn set_attribute_fails_on_missing_element() {
        let result = set_attribute(MASTER, "master.svg", "tint-unknown", "stop-color", "#DC2626");
        assert!(matches!(result, Err(ArtworkError::MissingElement { .. })));
    }

    #[test]
    fn set_attribute_fails_on_missing_attribute() {
        let result = set_attribute(MASTER, "master.svg", "tint-mid", "fill", "#DC2626");
        assert!(matches!(result, Err(ArtworkError::MissingAttribute { .. })));
    }

    #[test]
    fn set_text_replaces_content() {
        let result = set_text(MASTER, "master.svg", "rank-letter", "II").unwrap_or_default();
        assert_eq!(
            result,
            r##"<svg><stop id="tint-mid" offset="70%" stop-color="#2563EB" /><text id="rank-letter">II</text></svg>"##
        );
    }

    #[test]
    fn set_text_fails_on_non_text_element() {
        let result = set_text(MASTER, "master.svg", "tint-mid", "II");
        assert!(matches!(result, Err(ArtworkError::MissingElement { .. })));
    }

    #[test]
    fn set_inner_replaces_content() {
        let master = r##"<svg><g id="glyph"><circle r="1" /></g></svg>"##;
        let result = set_inner(master, "master.svg", "glyph", "<rect width=\"2\" height=\"2\" />").unwrap_or_default();
        assert_eq!(result, r##"<svg><g id="glyph"><rect width="2" height="2" /></g></svg>"##);
    }

    #[test]
    fn set_inner_handles_nested_same_name_elements() {
        let master = r##"<svg><g id="glyph"><g><g /></g><circle r="1" /></g><circle r="2" /></svg>"##;
        let result = set_inner(master, "master.svg", "glyph", "<path d=\"M0,0\" />").unwrap_or_default();
        assert_eq!(result, r##"<svg><g id="glyph"><path d="M0,0" /></g><circle r="2" /></svg>"##);
    }

    #[test]
    fn set_inner_rewrites_self_closing_target() {
        let master = r##"<svg><g id="glyph" transform="translate(0,1)" /></svg>"##;
        let result = set_inner(master, "master.svg", "glyph", "<circle r=\"1\" />").unwrap_or_default();
        assert_eq!(result, r##"<svg><g id="glyph" transform="translate(0,1)"><circle r="1" /></g></svg>"##);
    }

    #[test]
    fn set_inner_does_not_match_name_prefixes() {
        let master = r##"<svg><g id="glyph"><gradient /><circle r="1" /></g><circle r="2" /></svg>"##;
        let result = set_inner(master, "master.svg", "glyph", "X").unwrap_or_default();
        assert_eq!(result, r##"<svg><g id="glyph">X</g><circle r="2" /></svg>"##);
    }

    #[test]
    fn set_inner_fails_on_missing_element() {
        let result = set_inner(MASTER, "master.svg", "unknown", "X");
        assert!(matches!(result, Err(ArtworkError::MissingElement { .. })));
    }

    #[test]
    fn substitutions_skip_id_occurrences_inside_comments() {
        let master = r##"<!-- <text id="rank-letter"> doc --><svg><text id="rank-letter" fill="#fef0cd">I</text></svg>"##;
        let result = set_text(master, "master.svg", "rank-letter", "II").unwrap_or_default();
        assert_eq!(result, r##"<!-- <text id="rank-letter"> doc --><svg><text id="rank-letter" fill="#fef0cd">II</text></svg>"##);
        let result = set_attribute(master, "master.svg", "rank-letter", "fill", "#dfbe6f").unwrap_or_default();
        assert_eq!(result, r##"<!-- <text id="rank-letter"> doc --><svg><text id="rank-letter" fill="#dfbe6f">I</text></svg>"##);
    }
}
