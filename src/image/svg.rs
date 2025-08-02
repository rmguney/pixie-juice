//! SVG (Scalable Vector Graphics) optimization
//! Vector graphics format optimization

extern crate alloc;
use alloc::{vec::Vec, string::ToString, string::String, format};
use crate::types::{PixieResult, ImageOptConfig, PixieError};

/// Check if data is SVG format
pub fn is_svg(data: &[u8]) -> bool {
    if data.len() < 5 {
        return false;
    }
    
    // Check for SVG signature
    let text = core::str::from_utf8(data).unwrap_or("");
    
    // Look for <svg tag
    text.trim_start().starts_with("<?xml") && text.contains("<svg") ||
    text.trim_start().starts_with("<svg")
}

/// Optimize SVG image with advanced usvg/resvg support
#[cfg(feature = "codec-svg")]
pub fn optimize_svg(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate SVG format
    if !is_svg(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid SVG file".to_string()));
    }
    
    // For lossless mode with high quality, try to preserve original structure
    if config.lossless && quality > 90 {
        // Still perform basic optimization
        return optimize_svg_text(data);
    }
    
    // Try advanced SVG optimization with usvg
    #[cfg(target_arch = "wasm32")]
    {
        // For WASM, use text-based optimization as usvg may not be fully compatible
        optimize_svg_text(data)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // For native builds, try using usvg for advanced optimization
        use usvg::{Options, Tree};
        
        let svg_text = core::str::from_utf8(data)
            .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG UTF-8 error: {:?}", e)))?;
        
        // Parse SVG with usvg
        let opt = Options::default();
        match Tree::from_str(svg_text, &opt) {
            Ok(tree) => {
                // Re-serialize with optimizations
                let optimized_svg = tree.to_string(&usvg::WriteOptions::default());
                
                // Apply additional text-based optimizations
                let further_optimized = optimize_svg_text(optimized_svg.as_bytes())?;
                
                // Return the smaller version
                if further_optimized.len() < data.len() {
                    Ok(further_optimized)
                } else {
                    Ok(data.to_vec())
                }
            },
            Err(_) => {
                // Fall back to text-based optimization if usvg fails
                optimize_svg_text(data)
            }
        }
    }
}

/// Fallback for when SVG codec features are not available
#[cfg(not(feature = "codec-svg"))]
pub fn optimize_svg(data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate SVG format
    if !is_svg(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid SVG file".to_string()));
    }
    
    // Basic text-based optimization without external crates
    optimize_svg_text(data)
}

/// Convert SVG to raster format (PNG/JPEG) for aggressive optimization
pub fn convert_svg_to_raster(data: &[u8], quality: u8, _target_width: u32, _target_height: u32) -> PixieResult<Vec<u8>> {
    #[cfg(all(feature = "codec-svg", not(target_arch = "wasm32")))]
    {
        use usvg::{Options, Tree};
        
        let svg_text = core::str::from_utf8(data)
            .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG UTF-8 error: {:?}", e)))?;
        
        // Parse SVG with usvg for validation and basic processing
        let opt = Options::default();
        match Tree::from_str(svg_text, &opt) {
            Ok(_tree) => {
                // For now, return optimized SVG text instead of rasterization
                // TODO: Add proper rasterization with resvg when APIs are stabilized
                optimize_svg_text(data)
            },
            Err(_) => {
                // Fall back to text-based optimization if usvg fails
                optimize_svg_text(data)
            }
        }
    }
    
    #[cfg(not(all(feature = "codec-svg", not(target_arch = "wasm32"))))]
    {
        // Fallback: return optimized SVG text
        let _ = quality;
        optimize_svg_text(data)
    }
}

/// Check if SVG has transparency
fn has_transparency_in_svg(svg_text: &str) -> bool {
    svg_text.contains("opacity") || 
    svg_text.contains("fill-opacity") || 
    svg_text.contains("stroke-opacity") ||
    svg_text.contains("rgba") ||
    svg_text.contains("transparent")
}

/// Basic SVG optimization using text processing
fn optimize_svg_text(data: &[u8]) -> PixieResult<Vec<u8>> {
    let svg_text = core::str::from_utf8(data)
        .map_err(|e| PixieError::ProcessingError(format!("SVG UTF-8 error: {:?}", e)))?;
    
    let mut optimized = String::with_capacity(svg_text.len());
    let mut in_whitespace = false;
    let mut prev_char = ' ';
    
    for ch in svg_text.chars() {
        match ch {
            // Compress whitespace
            ' ' | '\t' | '\n' | '\r' => {
                if !in_whitespace && prev_char != '>' && prev_char != '<' {
                    optimized.push(' ');
                    in_whitespace = true;
                }
            }
            // Keep other characters
            _ => {
                optimized.push(ch);
                in_whitespace = false;
            }
        }
        prev_char = ch;
    }
    
    // Remove XML comments and unnecessary metadata
    let optimized = remove_svg_comments(&optimized);
    let optimized = remove_svg_metadata(&optimized);
    
    Ok(optimized.into_bytes())
}

/// Remove XML comments from SVG
fn remove_svg_comments(svg: &str) -> String {
    let mut result = String::with_capacity(svg.len());
    let mut chars = svg.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '<' && chars.peek() == Some(&'!') {
            // Check for comment start
            let mut temp = String::new();
            temp.push(ch);
            
            // Read ahead to see if it's a comment
            let mut ahead_chars = chars.clone();
            for _ in 0..3 {
                if let Some(next_ch) = ahead_chars.next() {
                    temp.push(next_ch);
                }
            }
            
            if temp.starts_with("<!--") {
                // Skip until comment end
                let mut comment_depth = 1;
                while comment_depth > 0 && chars.peek().is_some() {
                    let next_ch = chars.next().unwrap();
                    if next_ch == '-' && chars.peek() == Some(&'-') {
                        chars.next(); // consume second -
                        if chars.peek() == Some(&'>') {
                            chars.next(); // consume >
                            comment_depth -= 1;
                        }
                    }
                }
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }
    
    result
}

/// Remove unnecessary metadata from SVG
fn remove_svg_metadata(svg: &str) -> String {
    let lines: Vec<&str> = svg.lines().collect();
    let mut filtered_lines = Vec::new();
    let mut in_metadata = false;
    
    for line in lines {
        let trimmed = line.trim();
        
        // Skip XML declaration for very aggressive optimization
        if trimmed.starts_with("<?xml") {
            continue;
        }
        
        // Skip DOCTYPE declarations
        if trimmed.starts_with("<!DOCTYPE") {
            continue;
        }
        
        // Skip metadata tags
        if trimmed.starts_with("<metadata") || trimmed.starts_with("<title") || 
           trimmed.starts_with("<desc") {
            in_metadata = true;
        }
        
        if !in_metadata {
            filtered_lines.push(line);
        }
        
        if trimmed.contains("</metadata>") || trimmed.contains("</title>") || 
           trimmed.contains("</desc>") {
            in_metadata = false;
        }
    }
    
    filtered_lines.join("\n")
}

/// Get SVG metadata
pub fn get_svg_info(data: &[u8]) -> PixieResult<(u32, u32, u8)> {
    if !is_svg(data) {
        return Err(PixieError::InvalidFormat("Not a valid SVG file".to_string()));
    }
    
    let svg_text = core::str::from_utf8(data)
        .map_err(|e| PixieError::ProcessingError(format!("SVG UTF-8 error: {:?}", e)))?;
    
    // Try to extract width and height from SVG tag
    if let Some(svg_start) = svg_text.find("<svg") {
        let svg_tag_end = svg_text[svg_start..].find('>').unwrap_or(svg_text.len() - svg_start) + svg_start;
        let svg_tag = &svg_text[svg_start..svg_tag_end];
        
        let width = extract_svg_dimension(svg_tag, "width").unwrap_or(100);
        let height = extract_svg_dimension(svg_tag, "height").unwrap_or(100);
        
        return Ok((width, height, 8)); // SVG is vector, but report as 8-bit equivalent
    }
    
    // Default dimensions if not found
    Ok((100, 100, 8))
}

/// Extract dimension from SVG tag
fn extract_svg_dimension(svg_tag: &str, attr: &str) -> Option<u32> {
    if let Some(attr_start) = svg_tag.find(&format!("{}=\"", attr)) {
        let value_start = attr_start + attr.len() + 2;
        if let Some(value_end) = svg_tag[value_start..].find('"') {
            let value = &svg_tag[value_start..value_start + value_end];
            
            // Parse numeric value, handling units
            let numeric_part: String = value.chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            
            numeric_part.parse::<f32>().ok().map(|f| f as u32)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_svg_detection() {
        let svg_header = b"<svg xmlns=\"http://www.w3.org/2000/svg\">";
        assert!(is_svg(svg_header));
        
        let xml_svg = b"<?xml version=\"1.0\"?><svg>";
        assert!(is_svg(xml_svg));
        
        let not_svg = b"\x89PNG\r\n\x1a\n";
        assert!(!is_svg(not_svg));
    }
    
    #[test]
    fn test_svg_comment_removal() {
        let svg_with_comments = "<!-- This is a comment --><svg>content</svg>";
        let result = remove_svg_comments(svg_with_comments);
        assert_eq!(result, "<svg>content</svg>");
    }
}