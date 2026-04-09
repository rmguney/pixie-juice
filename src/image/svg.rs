extern crate alloc;
use alloc::{vec::Vec, string::{ToString, String}, format};
use crate::types::{PixieResult, ImageOptConfig, PixieError};

pub fn is_svg(data: &[u8]) -> bool {
    if data.len() < 5 {
        return false;
    }
    let text = core::str::from_utf8(data).unwrap_or("");
    let trimmed = text.trim_start();
    (trimmed.starts_with("<?xml") && text.contains("<svg")) || trimmed.starts_with("<svg")
}

#[cfg(feature = "codec-svg")]
pub fn optimize_svg(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    if !is_svg(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid SVG file".to_string()));
    }

    let result = optimize_svg_xml(data, quality, config)?;
    if result.len() < data.len() {
        Ok(result)
    } else {
        Ok(data.to_vec())
    }
}

#[cfg(not(feature = "codec-svg"))]
pub fn optimize_svg(data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    if !is_svg(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid SVG file".to_string()));
    }
    Ok(data.to_vec())
}

#[cfg(feature = "codec-svg")]
fn optimize_svg_xml(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
    use quick_xml::reader::Reader;
    use quick_xml::writer::Writer;

    let aggressive = quality <= 60 && !config.lossless;
    let strip_metadata = !config.preserve_metadata || aggressive;

    let mut reader = Reader::from_reader(data);
    reader.config_mut().trim_text(false);
    reader.config_mut().expand_empty_elements = false;
    reader.config_mut().check_end_names = false;

    let mut writer = Writer::new(Vec::with_capacity(data.len()));
    let mut buf = Vec::new();
    let mut skip_element_depth: i32 = 0;
    let mut skip_until_tag: Option<Vec<u8>> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(PixieError::ImageDecodingFailed(format!("SVG parse error: {}", e)));
            }
            Ok(Event::Eof) => break,
            Ok(event) => {
                if let Some(target) = &skip_until_tag {
                    if let Event::End(ref end) = event {
                        if end.name().as_ref() == target.as_slice() && skip_element_depth == 1 {
                            skip_until_tag = None;
                            skip_element_depth = 0;
                            buf.clear();
                            continue;
                        }
                    }
                    if let Event::Start(ref start) = event {
                        if start.name().as_ref() == target.as_slice() {
                            skip_element_depth += 1;
                        }
                    }
                    if let Event::End(ref end) = event {
                        if end.name().as_ref() == target.as_slice() {
                            skip_element_depth -= 1;
                            if skip_element_depth <= 0 {
                                skip_until_tag = None;
                                skip_element_depth = 0;
                            }
                        }
                    }
                    buf.clear();
                    continue;
                }

                match event {
                    Event::Decl(_) => {
                        if !strip_metadata {
                            let xml_decl = quick_xml::events::BytesDecl::new("1.0", Some("UTF-8"), None);
                            writer.write_event(Event::Decl(xml_decl))
                                .map_err(|e| PixieError::ProcessingError(format!("XML write error: {}", e)))?;
                        }
                    }
                    Event::DocType(_) => {
                        if !strip_metadata {
                            writer.write_event(event)
                                .map_err(|e| PixieError::ProcessingError(format!("XML write error: {}", e)))?;
                        }
                    }
                    Event::Comment(_) => {
                        // always strip comments
                    }
                    Event::PI(_) => {
                        if !strip_metadata {
                            writer.write_event(event)
                                .map_err(|e| PixieError::ProcessingError(format!("XML write error: {}", e)))?;
                        }
                    }
                    Event::Start(start) => {
                        let name_bytes = start.name().as_ref().to_vec();
                        if strip_metadata && is_metadata_tag(&name_bytes) {
                            skip_until_tag = Some(name_bytes);
                            skip_element_depth = 1;
                            buf.clear();
                            continue;
                        }
                        let cleaned = rewrite_start_tag(&start, aggressive)?;
                        writer.write_event(Event::Start(cleaned))
                            .map_err(|e| PixieError::ProcessingError(format!("XML write error: {}", e)))?;
                    }
                    Event::Empty(start) => {
                        let name_bytes = start.name().as_ref().to_vec();
                        if strip_metadata && is_metadata_tag(&name_bytes) {
                            buf.clear();
                            continue;
                        }
                        let cleaned = rewrite_start_tag(&start, aggressive)?;
                        writer.write_event(Event::Empty(cleaned))
                            .map_err(|e| PixieError::ProcessingError(format!("XML write error: {}", e)))?;
                    }
                    Event::End(end) => {
                        let name_bytes = end.name().as_ref().to_vec();
                        if strip_metadata && is_metadata_tag(&name_bytes) {
                            buf.clear();
                            continue;
                        }
                        writer.write_event(Event::End(BytesEnd::new(String::from_utf8_lossy(&name_bytes).to_string())))
                            .map_err(|e| PixieError::ProcessingError(format!("XML write error: {}", e)))?;
                    }
                    Event::Text(text) => {
                        let raw = text.into_inner();
                        let s = core::str::from_utf8(&raw).unwrap_or("");
                        if !s.chars().all(|c| c.is_whitespace()) {
                            writer.write_event(Event::Text(BytesText::from_escaped(s.to_string())))
                                .map_err(|e| PixieError::ProcessingError(format!("XML write error: {}", e)))?;
                        }
                    }
                    Event::CData(_) => {
                        writer.write_event(event)
                            .map_err(|e| PixieError::ProcessingError(format!("XML write error: {}", e)))?;
                    }
                    Event::Eof => break,
                }
            }
        }
        buf.clear();
    }

    Ok(writer.into_inner())
}

#[cfg(feature = "codec-svg")]
fn rewrite_start_tag<'a>(
    start: &quick_xml::events::BytesStart<'a>,
    aggressive: bool,
) -> PixieResult<quick_xml::events::BytesStart<'static>> {
    use quick_xml::events::attributes::Attribute;
    use quick_xml::events::BytesStart;

    let name_string = core::str::from_utf8(start.name().as_ref())
        .map_err(|_| PixieError::ImageDecodingFailed("SVG element name not valid UTF-8".to_string()))?
        .to_string();

    let mut out = BytesStart::new(name_string);

    for attr_result in start.attributes() {
        let attr = attr_result
            .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG attribute parse error: {}", e)))?;
        let key_owned = attr.key.as_ref().to_vec();
        let value = attr.unescape_value()
            .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG attribute unescape error: {}", e)))?
            .to_string();

        if aggressive && should_drop_attribute(&key_owned, &value) {
            continue;
        }

        let new_value = if is_color_attribute(&key_owned) {
            shorten_hex_color(&value)
        } else {
            value
        };

        let key_str = core::str::from_utf8(&key_owned)
            .map_err(|_| PixieError::ImageDecodingFailed("SVG attribute name not valid UTF-8".to_string()))?
            .to_string();
        out.push_attribute(Attribute {
            key: quick_xml::name::QName(key_str.as_bytes()),
            value: alloc::borrow::Cow::Owned(new_value.into_bytes()),
        });
    }

    Ok(out.into_owned())
}

fn is_metadata_tag(name: &[u8]) -> bool {
    let stripped = strip_xml_namespace(name);
    matches!(stripped, b"metadata" | b"title" | b"desc")
}

fn strip_xml_namespace(name: &[u8]) -> &[u8] {
    match name.iter().rposition(|&b| b == b':') {
        Some(idx) => &name[idx + 1..],
        None => name,
    }
}

fn is_color_attribute(key: &[u8]) -> bool {
    let stripped = strip_xml_namespace(key);
    matches!(
        stripped,
        b"fill" | b"stroke" | b"stop-color" | b"flood-color" | b"color" | b"lighting-color" | b"solid-color"
    )
}

fn should_drop_attribute(key: &[u8], value: &str) -> bool {
    let stripped = strip_xml_namespace(key);
    match stripped {
        b"version" => value == "1.0" || value == "1.1" || value == "1.2",
        b"xmlns:dc" | b"xmlns:cc" | b"xmlns:rdf" | b"xmlns:sodipodi" | b"xmlns:inkscape" => true,
        _ => false,
    }
}

fn shorten_hex_color(value: &str) -> String {
    let trimmed = value.trim();
    if !trimmed.starts_with('#') || trimmed.len() != 7 {
        return value.to_string();
    }
    let bytes = trimmed.as_bytes();
    let pairs = [(1, 2), (3, 4), (5, 6)];
    let mut compact = String::with_capacity(4);
    compact.push('#');
    for (a, b) in pairs {
        let ca = bytes[a].to_ascii_lowercase();
        let cb = bytes[b].to_ascii_lowercase();
        if ca != cb {
            return value.to_string();
        }
        compact.push(ca as char);
    }
    compact
}

pub fn convert_svg_to_raster(data: &[u8], _quality: u8, _target_width: u32, _target_height: u32) -> PixieResult<Vec<u8>> {
    if !is_svg(data) {
        return Err(PixieError::InvalidFormat("Not a valid SVG file".to_string()));
    }
    Ok(data.to_vec())
}

pub fn get_svg_info(data: &[u8]) -> PixieResult<(u32, u32, u8)> {
    if !is_svg(data) {
        return Err(PixieError::InvalidFormat("Not a valid SVG file".to_string()));
    }

    let svg_text = core::str::from_utf8(data)
        .map_err(|e| PixieError::ProcessingError(format!("SVG UTF-8 error: {:?}", e)))?;

    if let Some(svg_start) = svg_text.find("<svg") {
        let svg_tag_end = svg_text[svg_start..].find('>').map(|i| i + svg_start).unwrap_or(svg_text.len());
        let svg_tag = &svg_text[svg_start..svg_tag_end];

        let width = extract_svg_dimension(svg_tag, "width").unwrap_or(100);
        let height = extract_svg_dimension(svg_tag, "height").unwrap_or(100);

        return Ok((width, height, 8));
    }

    Ok((100, 100, 8))
}

fn extract_svg_dimension(svg_tag: &str, attr: &str) -> Option<u32> {
    let prefix = format!("{}=\"", attr);
    let attr_start = svg_tag.find(&prefix)?;
    let value_start = attr_start + prefix.len();
    let value_end = svg_tag[value_start..].find('"')?;
    let value = &svg_tag[value_start..value_start + value_end];

    let numeric_part: String = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    numeric_part.parse::<f32>().ok().map(|f| f as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_detection() {
        assert!(is_svg(b"<svg xmlns=\"http://www.w3.org/2000/svg\">"));
        assert!(is_svg(b"<?xml version=\"1.0\"?><svg>"));
        assert!(!is_svg(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn test_shorten_hex_color() {
        assert_eq!(shorten_hex_color("#000000"), "#000");
        assert_eq!(shorten_hex_color("#ffffff"), "#fff");
        assert_eq!(shorten_hex_color("#aabbcc"), "#abc");
        assert_eq!(shorten_hex_color("#123456"), "#123456");
        assert_eq!(shorten_hex_color("red"), "red");
    }

    #[test]
    fn test_metadata_tag_detection() {
        assert!(is_metadata_tag(b"metadata"));
        assert!(is_metadata_tag(b"title"));
        assert!(is_metadata_tag(b"desc"));
        assert!(is_metadata_tag(b"svg:title"));
        assert!(!is_metadata_tag(b"path"));
    }
}
