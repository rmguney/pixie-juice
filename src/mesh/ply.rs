//! PLY format support

extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}, format};

use crate::types::{MeshOptConfig, OptResult, OptError};

/// Optimize PLY format
pub fn optimize_ply(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    // Simple text-based PLY optimization
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY file contains invalid UTF-8".to_string()))?;
    
    if !content.starts_with("ply") {
        return Err(OptError::InvalidFormat("Not a valid PLY file".to_string()));
    }
    
    let optimized_content = if config.weld_vertices {
        // Remove duplicate vertex data and compress precision
        optimize_ply_geometry(content, config)?
    } else {
        // Minimal optimization - remove comments and extra whitespace
        remove_ply_comments(content)?
    };
    
    Ok(optimized_content.into_bytes())
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
        
        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }
        
        // Track vertex count from header
        if trimmed.starts_with("element vertex ") {
            if let Some(count_str) = trimmed.split_whitespace().nth(2) {
                vertex_count = count_str.parse().unwrap_or(0);
            }
            result_lines.push(line.to_string());
            continue;
        }
        
        // Detect end of header
        if trimmed == "end_header" {
            in_vertex_data = true;
            result_lines.push(line.to_string());
            continue;
        }
        
        // Process vertex data
        if in_vertex_data && processed_vertices < vertex_count {
            let optimized_vertex = optimize_vertex_line(line, config)?;
            result_lines.push(optimized_vertex);
            processed_vertices += 1;
            
            if processed_vertices >= vertex_count {
                in_vertex_data = false;
            }
            continue;
        }
        
        // Copy other lines as-is
        result_lines.push(line.to_string());
    }
    
    Ok(result_lines.join("\n"))
}

fn optimize_vertex_line(line: &str, config: &MeshOptConfig) -> OptResult<String> {
    let values: Vec<&str> = line.split_whitespace().collect();
    let mut optimized_values = Vec::new();
    
    for value in values {
        if let Ok(f) = value.parse::<f32>() {
            // Reduce precision based on vertex tolerance
            let factor = 1.0 / config.vertex_tolerance;
            let rounded = (f * factor).round() / factor;
            optimized_values.push(format!("{:.6}", rounded).trim_end_matches('0').trim_end_matches('.').to_string());
        } else {
            optimized_values.push(value.to_string());
        }
    }
    
    Ok(optimized_values.join(" "))
}

/// Validate PLY file structure
pub fn validate_ply_structure(data: &[u8]) -> OptResult<bool> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY file contains invalid UTF-8".to_string()))?;
    
    if !content.starts_with("ply") {
        return Err(OptError::InvalidFormat("Not a valid PLY file".to_string()));
    }
    
    // Check for required header elements
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
        
        // Simple PLY content for testing
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
        // Should not contain unnecessary precision
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
