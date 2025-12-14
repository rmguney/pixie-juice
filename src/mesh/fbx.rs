extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}};

use crate::types::{MeshOptConfig, OptResult, OptError};

const FBX_BINARY_SIGNATURE: &[u8] = b"Kaydara FBX Binary  \x00\x1a\x00";
const FBX_ASCII_SIGNATURE: &str = "; FBX";

pub fn optimize_fbx(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if is_binary_fbx(data) {
        optimize_binary_fbx(data, config)
    } else if is_ascii_fbx(data) {
        let normalized = {
            #[cfg(c_hotspots_available)]
            {
                if !config.preserve_topology && config.target_ratio < 1.0 {
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
            .map_err(|_| OptError::InvalidFormat("FBX ASCII contains invalid UTF-8".to_string()))?;
        
        let optimized_content = optimize_fbx_ascii(content, config)?;
        Ok(optimized_content.into_bytes())
    } else {
        Err(OptError::InvalidFormat("Not a valid FBX file".to_string()))
    }
}

fn is_binary_fbx(data: &[u8]) -> bool {
    data.len() >= 23 && data.starts_with(FBX_BINARY_SIGNATURE)
}

fn is_ascii_fbx(data: &[u8]) -> bool {
    if data.len() < 10 {
        return false;
    }
    
    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 200)]) {
        if text.starts_with(FBX_ASCII_SIGNATURE) {
            return true;
        }
        
        let text_lower = text.to_lowercase();
        return text_lower.contains("fbx") && 
               (text_lower.contains("objects:") || 
                text_lower.contains("fbxheaderextension:") ||
                text_lower.contains("model:") ||
                text_lower.contains("geometry:"));
    }
    
    false
}

fn optimize_binary_fbx(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if !config.preserve_topology {
        let mut result = data.to_vec();
        
        while result.last() == Some(&0) {
            result.pop();
        }
        
        if result.len() < 27 {
            return Ok(data.to_vec());
        }
        
        Ok(result)
    } else {
        Ok(data.to_vec())
    }
}

fn optimize_fbx_ascii(content: &str, config: &MeshOptConfig) -> OptResult<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut optimized_lines = Vec::<String>::new();
    
    for line in lines {
        let trimmed = line.trim();
        
        if config.preserve_topology {
            optimized_lines.push(line.to_string());
            continue;
        }
        
        if trimmed.starts_with(';') {
            if trimmed.contains("FBX") || trimmed.contains("Creator") {
                optimized_lines.push(line.to_string());
            }
            continue;
        }
        
        if trimmed.is_empty() {
            continue;
        }
        
        if config.target_ratio < 1.0 {
            let compressed = compress_fbx_line(line);
            optimized_lines.push(compressed);
        } else {
            optimized_lines.push(line.to_string());
        }
    }
    
    Ok(optimized_lines.join("\n"))
}

fn compress_fbx_line(line: &str) -> String {
    let trimmed = line.trim();
    
    if trimmed.contains(',') && (trimmed.contains('.') || trimmed.chars().any(|c| c.is_numeric())) {
        trimmed.split(',').map(|s| s.trim()).collect::<Vec<_>>().join(",")
    } else {
        trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

pub fn validate_fbx_structure(data: &[u8]) -> OptResult<bool> {
    if data.len() < 23 {
        return Err(OptError::InvalidFormat("FBX file too small".to_string()));
    }
    
    if data.starts_with(FBX_BINARY_SIGNATURE) {
        return Ok(true);
    }
    
    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 100)]) {
        if text.starts_with(FBX_ASCII_SIGNATURE) {
            let content = core::str::from_utf8(data)
                .map_err(|_| OptError::InvalidFormat("FBX ASCII contains invalid UTF-8".to_string()))?;
            
            if content.contains("FBX") && (content.contains("Objects:") || content.contains("Model:")) {
                return Ok(true);
            }
        }
    }
    
    Err(OptError::InvalidFormat("Not a valid FBX file".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fbx_signature_detection() {
        let binary_fbx_header = [
            b"Kaydara FBX Binary  \x00\x1a\x00".to_vec(),
            alloc::vec![0x00, 0x00],
            alloc::vec![0xE8, 0x1C, 0x00, 0x00],
        ].concat();
        
        assert!(binary_fbx_header.starts_with(FBX_BINARY_SIGNATURE));
        
        let ascii_fbx = b"; FBX 7.4.0 project file";
        assert!(ascii_fbx.starts_with(FBX_ASCII_SIGNATURE.as_bytes()));
    }
    
    #[test]
    fn test_fbx_validation() {
        let ascii_fbx = b"; FBX 7.4.0 project file
; Created by test
FBXHeaderExtension:  {
    FBXHeaderVersion: 1003
}
Objects:  {
    Model: 1234, \"Cube\", \"Mesh\" {
    }
}
";
        
        let result = validate_fbx_structure(ascii_fbx);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        let invalid_fbx = b"not_an_fbx_file";
        let result = validate_fbx_structure(invalid_fbx);
        assert!(result.is_err());
    }
}
