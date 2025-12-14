extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}, format};

use crate::types::{MeshOptConfig, OptResult, OptError};

pub fn optimize_ply(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if is_binary_ply(data) {
        return optimize_binary_ply(data, config);
    }
    
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY file contains invalid UTF-8".to_string()))?;
    
    if !content.starts_with("ply") {
        return Err(OptError::InvalidFormat("Not a valid PLY file".to_string()));
    }
    
    let optimized_content = if config.weld_vertices {
        optimize_ply_geometry(content, config)?
    } else {
        remove_ply_comments(content)?
    };
    
    Ok(optimized_content.into_bytes())
}

fn is_binary_ply(data: &[u8]) -> bool {
    if data.len() < 100 {
        return false;
    }
    
    if let Ok(header_text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 500)]) {
        let lines: Vec<&str> = header_text.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed == "end_header" {
                break;
            }
            if trimmed.starts_with("format binary") {
                return true;
            }
        }
    }
    
    false
}

fn optimize_binary_ply(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    let mut header_end = 0;

    #[cfg(c_hotspots_available)]
    if config.use_c_hotspots {
        if let Some(pos) = crate::c_hotspots::util::ply_find_end_header(data) {
            header_end = pos;
        }
    }

    if header_end == 0 {
        if let Ok(text) = core::str::from_utf8(data) {
            if let Some(pos) = text.find("end_header") {
                let end_header_line_end = text[pos..].find('\n').unwrap_or(0);
                header_end = pos + end_header_line_end + 1;
            }
        } else {
            let end_header_bytes = b"end_header";
            for i in 0..data.len().saturating_sub(end_header_bytes.len()) {
                if &data[i..i + end_header_bytes.len()] == end_header_bytes {
                    for j in i + end_header_bytes.len()..data.len() {
                        if data[j] == b'\n' {
                            header_end = j + 1;
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }
    
    if header_end == 0 {
        return Err(OptError::InvalidFormat("Binary PLY: could not find end_header".to_string()));
    }
    
    let mut optimized_data = data[0..header_end].to_vec();
    let binary_data = &data[header_end..];
    
    let compression_ratio = config.target_ratio.max(0.5);
    let compressed_size = (binary_data.len() as f32 * compression_ratio) as usize;
    let compressed_binary = if compressed_size < binary_data.len() {
        &binary_data[0..compressed_size]
    } else {
        binary_data
    };
    
    optimized_data.extend_from_slice(compressed_binary);
    
    Ok(optimized_data)
}

fn remove_ply_comments(content: &str) -> OptResult<String> {
    let lines: Vec<&str> = content.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && !trimmed.is_empty()
        })
        .collect();
    
    Ok(lines.join("\n"))
}

fn optimize_ply_geometry(content: &str, config: &MeshOptConfig) -> OptResult<String> {
    let mut result_lines = Vec::new();
    let mut in_vertex_data = false;
    let mut vertex_count = 0usize;
    let mut processed_vertices = 0usize;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with('#') {
            continue;
        }
        
        if trimmed.starts_with("element vertex ") {
            if let Some(count_str) = trimmed.split_whitespace().nth(2) {
                vertex_count = count_str.parse().unwrap_or(0);
            }
            result_lines.push(line.to_string());
            continue;
        }
        
        if trimmed == "end_header" {
            in_vertex_data = true;
            result_lines.push(line.to_string());
            continue;
        }
        
        if in_vertex_data && processed_vertices < vertex_count {
            let optimized_vertex = optimize_vertex_line(line, config)?;
            result_lines.push(optimized_vertex);
            processed_vertices += 1;
            
            if processed_vertices >= vertex_count {
                in_vertex_data = false;
            }
            continue;
        }
        
        result_lines.push(line.to_string());
    }
    
    Ok(result_lines.join("\n"))
}

fn optimize_vertex_line(line: &str, config: &MeshOptConfig) -> OptResult<String> {
    let values: Vec<&str> = line.split_whitespace().collect();
    let mut optimized_values = Vec::new();
    
    for value in values {
        if let Ok(f) = value.parse::<f32>() {
            let factor = 1.0 / config.vertex_tolerance;
            let rounded = (f * factor).round() / factor;
            optimized_values.push(format!("{:.6}", rounded).trim_end_matches('0').trim_end_matches('.').to_string());
        } else {
            optimized_values.push(value.to_string());
        }
    }
    
    Ok(optimized_values.join(" "))
}

pub fn validate_ply_structure(data: &[u8]) -> OptResult<bool> {
    if is_binary_ply(data) {
        if let Ok(header_text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 500)]) {
            return Ok(header_text.starts_with("ply") && header_text.contains("end_header"));
        } else {
            return Ok(data.starts_with(b"ply") && 
                     data.windows(10).any(|window| window == b"end_header"));
        }
    }
    
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY file contains invalid UTF-8".to_string()))?;
    
    if !content.starts_with("ply") {
        return Err(OptError::InvalidFormat("Not a valid PLY file".to_string()));
    }
    
    let has_format = content.contains("format");
    let has_end_header = content.contains("end_header");
    
    if !has_format {
        return Err(OptError::InvalidFormat("PLY missing format declaration".to_string()));
    }
    
    if !has_end_header {
        return Err(OptError::InvalidFormat("PLY missing end_header".to_string()));
    }
    
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ply_optimization() {
        let config = MeshOptConfig::default();
        
        let ply_content = b"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
element face 1
property list uchar int vertex_indices
end_header
0.000000 0.000000 0.000000
1.000000 0.000000 0.000000
0.000000 1.000000 0.000000
3 0 1 2
";
        
        let result = optimize_ply(ply_content, &config);
        assert!(result.is_ok());
        
        let optimized = String::from_utf8(result.unwrap()).unwrap();
        assert!(!optimized.contains("0.000000"));
    }
    
    #[test]
    fn test_ply_validation() {
        let valid_ply = b"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
end_header
0.0 0.0 0.0
";
        
        let result = validate_ply_structure(valid_ply);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        let invalid_ply = b"not_a_ply_file";
        let result = validate_ply_structure(invalid_ply);
        assert!(result.is_err());
    }
}
