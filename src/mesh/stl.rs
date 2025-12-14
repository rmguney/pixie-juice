extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}, format};

use crate::types::{MeshOptConfig, OptResult, OptError};
use crate::optimizers::get_current_time_ms;

pub fn optimize_stl(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    let start_time = get_current_time_ms();
    let data_size = data.len();
    
    #[cfg(feature = "fmt-stl")]
    {
        optimize_stl_with_crate(data, config, start_time, data_size)
    }
    
    #[cfg(not(feature = "fmt-stl"))]
    {
        optimize_stl_text_only(data, config, start_time, data_size)
    }
}

#[cfg(feature = "fmt-stl")]
fn optimize_stl_with_crate(data: &[u8], config: &MeshOptConfig, start_time: f64, data_size: usize) -> OptResult<Vec<u8>> {
    optimize_stl_text_only(data, config, start_time, data_size)
}


fn optimize_stl_text_only(data: &[u8], config: &MeshOptConfig, start_time: f64, data_size: usize) -> OptResult<Vec<u8>> {
    if data.starts_with(b"solid ") {
        optimize_ascii_stl_text(data, config, start_time, data_size)
    } else {
        optimize_binary_stl_basic(data, config, start_time, data_size)
    }
}

fn optimize_binary_stl_basic(data: &[u8], config: &MeshOptConfig, _start_time: f64, _data_size: usize) -> OptResult<Vec<u8>> {
    let mut result = data.to_vec();
    
    if config.target_ratio < 1.0 {
        let target_size = (data.len() as f32 * config.target_ratio) as usize;
        result.truncate(target_size.max(84));
    }
    
    Ok(result)
}

fn optimize_ascii_stl_text(data: &[u8], config: &MeshOptConfig, _start_time: f64, _data_size: usize) -> OptResult<Vec<u8>> {
    let normalized = {
        #[cfg(c_hotspots_available)]
        {
            if config.target_ratio < 1.0 {
                if let Some(out) = crate::c_hotspots::util::normalize_text_whitespace_commas(data) {
                    out
                } else {
                    data.to_vec()
                }
            } else {
                data.to_vec()
            }
        }

        #[cfg(not(c_hotspots_available))]
        {
            data.to_vec()
        }
    };

    let content = core::str::from_utf8(&normalized)
        .map_err(|_| OptError::InvalidFormat("STL file contains invalid UTF-8".to_string()))?;
    
    let lines: Vec<&str> = content.lines().collect();
    let mut optimized_lines = Vec::new();
    let precision = if config.target_ratio < 0.5 { 3 } else { 6 };
    
    for line in lines {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            continue;
        }
        
        if trimmed.starts_with("vertex ") {
            if let Ok(optimized_vertex) = optimize_stl_vertex_line(trimmed, precision) {
                optimized_lines.push(optimized_vertex);
            } else {
                optimized_lines.push(trimmed.to_string());
            }
        } else if trimmed.starts_with("facet normal ") {
            if let Ok(optimized_normal) = optimize_stl_normal_line(trimmed, precision) {
                optimized_lines.push(optimized_normal);
            } else {
                optimized_lines.push(trimmed.to_string());
            }
        } else {
            optimized_lines.push(trimmed.to_string());
        }
    }
    
    Ok(optimized_lines.join("\n").into_bytes())
}

fn optimize_stl_vertex_line(line: &str, precision: usize) -> OptResult<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 4 || parts[0] != "vertex" {
        return Err(OptError::ProcessingError("Invalid vertex line".to_string()));
    }
    
    let x: f32 = parts[1].parse()
        .map_err(|_| OptError::ProcessingError("Invalid vertex coordinate".to_string()))?;
    let y: f32 = parts[2].parse()
        .map_err(|_| OptError::ProcessingError("Invalid vertex coordinate".to_string()))?;
    let z: f32 = parts[3].parse()
        .map_err(|_| OptError::ProcessingError("Invalid vertex coordinate".to_string()))?;
    
    Ok(format!("vertex {:.prec$} {:.prec$} {:.prec$}", x, y, z, prec = precision))
}

fn optimize_stl_normal_line(line: &str, precision: usize) -> OptResult<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 5 || parts[0] != "facet" || parts[1] != "normal" {
        return Err(OptError::ProcessingError("Invalid normal line".to_string()));
    }
    
    let x: f32 = parts[2].parse()
        .map_err(|_| OptError::ProcessingError("Invalid normal coordinate".to_string()))?;
    let y: f32 = parts[3].parse()
        .map_err(|_| OptError::ProcessingError("Invalid normal coordinate".to_string()))?;
    let z: f32 = parts[4].parse()
        .map_err(|_| OptError::ProcessingError("Invalid normal coordinate".to_string()))?;
    
    Ok(format!("facet normal {:.prec$} {:.prec$} {:.prec$}", x, y, z, prec = precision))
}

pub fn is_stl(data: &[u8]) -> bool {
    if data.starts_with(b"solid ") {
        return true;
    }
    
    if data.len() >= 84 {
        let triangle_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]);
        let expected_size = 84 + (triangle_count as usize * 50);
        data.len() == expected_size
    } else {
        false
    }
}

pub fn validate_stl_structure(data: &[u8]) -> OptResult<bool> {
    if data.len() < 15 {
        return Err(OptError::InvalidFormat("STL file too small".to_string()));
    }
    
    if data.starts_with(b"solid ") {
        let content = core::str::from_utf8(data)
            .map_err(|_| OptError::InvalidFormat("STL file contains invalid UTF-8".to_string()))?;
        
        if !content.contains("facet normal") || !content.contains("vertex") {
            return Err(OptError::InvalidFormat("Invalid STL structure".to_string()));
        }
        
        return Ok(true);
    }
    
    if data.len() >= 84 {
        let triangle_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]);
        let expected_size = 84 + (triangle_count as usize * 50);
        if data.len() == expected_size {
            return Ok(true);
        }
    }
    
    Err(OptError::InvalidFormat("Not a valid STL file".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stl_optimization() {
        let config = MeshOptConfig::default();
        
        let ascii_stl = b"solid test
facet normal 0.000000 0.000000 1.000000
  outer loop
    vertex 0.000000 0.000000 0.000000
    vertex 1.000000 0.000000 0.000000
    vertex 0.000000 1.000000 0.000000
  endloop
endfacet
endsolid test
";
        
        let result = optimize_stl(ascii_stl, &config);
        assert!(result.is_ok());
        
        let optimized = String::from_utf8(result.unwrap()).unwrap();
        assert!(!optimized.contains("0.000000"));
    }
    
    #[test]
    fn test_stl_validation() {
        let valid_ascii_stl = b"solid test
facet normal 0 0 1
  outer loop
    vertex 0 0 0
    vertex 1 0 0
    vertex 0 1 0
  endloop
endfacet
endsolid test
";
        
        let result = validate_stl_structure(valid_ascii_stl);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        let invalid_stl = b"not_an_stl_file";
        let result = validate_stl_structure(invalid_stl);
        assert!(result.is_err());
    }
}
