extern crate alloc;
use alloc::{vec::Vec, string::ToString, string::String, format};
use crate::types::{PixieResult, ImageOptConfig, PixieError};

pub fn is_svg(data: &[u8]) -> bool {
    if data.len() < 5 {
        return false;
    }
    
    let text = core::str::from_utf8(data).unwrap_or("");
    
    text.trim_start().starts_with("<?xml") && text.contains("<svg") ||
    text.trim_start().starts_with("<svg")
}

#[cfg(feature = "codec-svg")]
pub fn optimize_svg(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    if !is_svg(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid SVG file".to_string()));
    }
    
    if config.lossless && quality > 90 {
        return optimize_svg_conservative(data);
    }
    
    let preprocessed = apply_svg_c_hotspot_preprocessing(data, quality)?;
    
    let strategies = get_svg_optimization_strategies(quality);
    let mut best_result = preprocessed;
    let mut best_size = best_result.len();
    
    for strategy in strategies {
        if let Ok(optimized) = apply_svg_optimization_strategy(&best_result, strategy) {
            if optimized.len() < best_size {
                best_result = optimized;
                best_size = best_result.len();
            }
        }
    }
    
    if best_result.len() < data.len() {
        Ok(best_result)
    } else {
        Ok(data.to_vec())
    }
}

fn apply_svg_c_hotspot_preprocessing(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        let mut current_data = data.to_vec();
        
        if quality <= 70 {
            current_data = crate::c_hotspots::svg_text_compress(&current_data)?;
        }
        
        if quality <= 80 {
            current_data = crate::c_hotspots::svg_minify_markup(&current_data)?;
        }
        
        if quality <= 60 {
            current_data = crate::c_hotspots::svg_optimize_paths_c(&current_data)?;
        }
        
        Ok(current_data)
    }
    #[cfg(not(c_hotspots_available))]
    {
        let svg_text = core::str::from_utf8(data)
            .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG UTF-8 error: {:?}", e)))?;
        optimize_svg_text_fallback(svg_text, quality)
    }
}

fn optimize_svg_text_fallback(svg_text: &str, quality: u8) -> PixieResult<Vec<u8>> {
    let mut result = svg_text.to_string();
    
    result = result.split("<!--").collect::<Vec<_>>().into_iter()
        .enumerate()
        .filter_map(|(i, part)| {
            if i == 0 {
                Some(part.to_string())
            } else if let Some(end) = part.find("-->") {
                Some(part[end + 3..].to_string())
            } else {
                None
            }
        })
        .collect::<String>();
    
    if quality <= 60 {
        result = result.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    
    Ok(result.into_bytes())
}

#[cfg(not(feature = "codec-svg"))]
pub fn optimize_svg(data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    if !is_svg(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid SVG file".to_string()));
    }
    
    optimize_svg_text(data)
}

#[derive(Debug, Clone)]
enum SvgOptimizationStrategy {
    CleanupMetadata,
    RemoveUnused,
    OptimizePaths,
    OptimizeColors,
    MergeDuplicates,
    AggressiveOptimization,
}

fn get_svg_optimization_strategies(quality: u8) -> Vec<SvgOptimizationStrategy> {
    let mut strategies = Vec::new();
    
    strategies.push(SvgOptimizationStrategy::CleanupMetadata);
    
    if quality <= 80 {
        strategies.push(SvgOptimizationStrategy::RemoveUnused);
        strategies.push(SvgOptimizationStrategy::OptimizeColors);
    }
    
    if quality <= 60 {
        strategies.push(SvgOptimizationStrategy::OptimizePaths);
        strategies.push(SvgOptimizationStrategy::MergeDuplicates);
    }
    
    if quality <= 40 {
        strategies.push(SvgOptimizationStrategy::AggressiveOptimization);
    }
    
    strategies
}

fn apply_svg_optimization_strategy(data: &[u8], strategy: SvgOptimizationStrategy) -> PixieResult<Vec<u8>> {
    let svg_text = core::str::from_utf8(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG UTF-8 error: {:?}", e)))?;
    
    match strategy {
        SvgOptimizationStrategy::CleanupMetadata => cleanup_svg_metadata(svg_text),
        SvgOptimizationStrategy::RemoveUnused => remove_unused_elements(svg_text),
        SvgOptimizationStrategy::OptimizePaths => optimize_svg_paths(svg_text),
        SvgOptimizationStrategy::OptimizeColors => optimize_svg_colors(svg_text),
        SvgOptimizationStrategy::MergeDuplicates => merge_duplicate_elements(svg_text),
        SvgOptimizationStrategy::AggressiveOptimization => aggressive_svg_optimization(svg_text),
    }
}

fn optimize_svg_conservative(data: &[u8]) -> PixieResult<Vec<u8>> {
    let svg_text = core::str::from_utf8(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG UTF-8 error: {:?}", e)))?;
    
    cleanup_svg_metadata(svg_text)
}

fn cleanup_svg_metadata(svg_text: &str) -> PixieResult<Vec<u8>> {
    let mut result = String::with_capacity(svg_text.len());
    let mut i = 0;
    let bytes = svg_text.as_bytes();
    
    while i < bytes.len() {
          if bytes[i] == b'<' && i + 4 < bytes.len() &&
              &bytes[i..i+4] == b"<!--" {
            i += 4;
            while i + 2 < bytes.len() {
                if &bytes[i..i+3] == b"-->" {
                    i += 3;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        let ch = bytes[i] as char;
        
        if ch.is_whitespace() {
            if !result.chars().last().map_or(false, |c| c.is_whitespace()) {
                result.push(' ');
            }
        } else {
            result.push(ch);
        }
        
        i += 1;
    }
    
    Ok(result.trim().as_bytes().to_vec())
}

fn remove_unused_elements(svg_text: &str) -> PixieResult<Vec<u8>> {
    let mut result = svg_text.to_string();
    
    result = result.replace("<defs></defs>", "");
    result = result.replace("<defs/>", "");
    
    result = result.replace("<g></g>", "");
    result = result.replace("<g/>", "");
    
    Ok(result.as_bytes().to_vec())
}

fn optimize_svg_paths(svg_text: &str) -> PixieResult<Vec<u8>> {
    let mut result = svg_text.to_string();
    
    result = result.replace(" ,", ",");
    result = result.replace(", ", ",");
    result = result.replace("  ", " ");
    
    
    Ok(result.as_bytes().to_vec())
}

fn optimize_svg_colors(svg_text: &str) -> PixieResult<Vec<u8>> {
    let mut result = svg_text.to_string();
    
    result = result.replace("#000000", "#000");
    result = result.replace("#ffffff", "#fff");
    result = result.replace("#ff0000", "#f00");
    result = result.replace("#00ff00", "#0f0");
    result = result.replace("#0000ff", "#00f");
    
    
    Ok(result.as_bytes().to_vec())
}

fn merge_duplicate_elements(svg_text: &str) -> PixieResult<Vec<u8>> {
    let result = svg_text.to_string();
    
    
    Ok(result.as_bytes().to_vec())
}

fn aggressive_svg_optimization(svg_text: &str) -> PixieResult<Vec<u8>> {
    let mut result = svg_text.to_string();
    
    while let Some(start) = result.find("<!--") {
        if let Some(end) = result[start..].find("-->") {
            result.replace_range(start..start + end + 3, "");
        } else {
            break;
        }
    }
    
    result = result.replace(" xmlns=\"http://www.w3.org/2000/svg\"", "");
    result = result.replace(" version=\"1.1\"", "");
    
    result = result.replace("\n", "").replace("\t", "").replace("  ", " ");
    
    Ok(result.trim().as_bytes().to_vec())
}

pub fn convert_svg_to_raster(data: &[u8], quality: u8, _target_width: u32, _target_height: u32) -> PixieResult<Vec<u8>> {
    #[cfg(all(feature = "codec-svg", not(target_arch = "wasm32")))]
    {
        let svg_text = core::str::from_utf8(data)
            .map_err(|e| PixieError::ImageDecodingFailed(format!("SVG UTF-8 error: {:?}", e)))?;
        
        if quality <= 50 {
            aggressive_svg_optimization(svg_text)
        } else {
            optimize_svg_text(data)
        }
    }
    
    #[cfg(not(all(feature = "codec-svg", not(target_arch = "wasm32"))))]
    {
        let _ = quality;
        optimize_svg_text(data)
    }
}

fn has_transparency_in_svg(svg_text: &str) -> bool {
    svg_text.contains("opacity") || 
    svg_text.contains("fill-opacity") || 
    svg_text.contains("stroke-opacity") ||
    svg_text.contains("rgba") ||
    svg_text.contains("transparent")
}

fn optimize_svg_text(data: &[u8]) -> PixieResult<Vec<u8>> {
    let svg_text = core::str::from_utf8(data)
        .map_err(|e| PixieError::ProcessingError(format!("SVG UTF-8 error: {:?}", e)))?;
    
    let mut optimized = String::with_capacity(svg_text.len());
    let mut in_whitespace = false;
    let mut prev_char = ' ';
    
    for ch in svg_text.chars() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                if !in_whitespace && prev_char != '>' && prev_char != '<' {
                    optimized.push(' ');
                    in_whitespace = true;
                }
            }
            _ => {
                optimized.push(ch);
                in_whitespace = false;
            }
        }
        prev_char = ch;
    }
    
    let optimized = remove_svg_comments(&optimized);
    let optimized = remove_svg_metadata(&optimized);
    
    Ok(optimized.into_bytes())
}

fn remove_svg_comments(svg: &str) -> String {
    let mut result = String::with_capacity(svg.len());
    let mut chars = svg.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '<' && chars.peek() == Some(&'!') {
            let mut temp = String::new();
            temp.push(ch);
            
            let mut ahead_chars = chars.clone();
            for _ in 0..3 {
                if let Some(next_ch) = ahead_chars.next() {
                    temp.push(next_ch);
                }
            }
            
            if temp.starts_with("<!--") {
                let mut comment_depth = 1;
                while comment_depth > 0 && chars.peek().is_some() {
                    let next_ch = chars.next().unwrap();
                    if next_ch == '-' && chars.peek() == Some(&'-') {
                        chars.next();
                        if chars.peek() == Some(&'>') {
                            chars.next();
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

fn remove_svg_metadata(svg: &str) -> String {
    let lines: Vec<&str> = svg.lines().collect();
    let mut filtered_lines = Vec::new();
    let mut in_metadata = false;
    
    for line in lines {
        let trimmed = line.trim();
        
        if trimmed.starts_with("<?xml") {
            continue;
        }
        
        if trimmed.starts_with("<!DOCTYPE") {
            continue;
        }
        
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

pub fn get_svg_info(data: &[u8]) -> PixieResult<(u32, u32, u8)> {
    if !is_svg(data) {
        return Err(PixieError::InvalidFormat("Not a valid SVG file".to_string()));
    }
    
    let svg_text = core::str::from_utf8(data)
        .map_err(|e| PixieError::ProcessingError(format!("SVG UTF-8 error: {:?}", e)))?;
    
    if let Some(svg_start) = svg_text.find("<svg") {
        let svg_tag_end = svg_text[svg_start..].find('>').unwrap_or(svg_text.len() - svg_start) + svg_start;
        let svg_tag = &svg_text[svg_start..svg_tag_end];
        
        let width = extract_svg_dimension(svg_tag, "width").unwrap_or(100);
        let height = extract_svg_dimension(svg_tag, "height").unwrap_or(100);
        
        return Ok((width, height, 8));
    }
    
    Ok((100, 100, 8))
}

fn extract_svg_dimension(svg_tag: &str, attr: &str) -> Option<u32> {
    if let Some(attr_start) = svg_tag.find(&format!("{}=\"", attr)) {
        let value_start = attr_start + attr.len() + 2;
        if let Some(value_end) = svg_tag[value_start..].find('"') {
            let value = &svg_tag[value_start..value_start + value_end];
            
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