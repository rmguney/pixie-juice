//! FBX format support

extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}};

use crate::types::{MeshOptConfig, OptResult, OptError};

/// FBX file header signature
const FBX_BINARY_SIGNATURE: &[u8] = b"Kaydara FBX Binary  \x00\x1a\x00";
const FBX_ASCII_SIGNATURE: &[u8] = b"; FBX ";

/// Optimize FBX format
pub fn optimize_fbx(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    // Detect FBX format (binary vs ASCII)
    if data.starts_with(FBX_BINARY_SIGNATURE) {
        // Binary FBX - for now, return as-is due to complexity
        // TODO: Implement binary FBX optimization when proper parsing is available
        Ok(data.to_vec())
    } else if data.starts_with(FBX_ASCII_SIGNATURE) {
        // ASCII FBX - simpler text processing
        let content = core::str::from_utf8(data)
            .map_err(|_| OptError::InvalidFormat("FBX ASCII contains invalid UTF-8".to_string()))?;
        
        let optimized_content = optimize_fbx_ascii(content, config)?;
        Ok(optimized_content.into_bytes())
    } else {
        Err(OptError::InvalidFormat("Not a valid FBX file".to_string()))
    }
}

fn optimize_fbx_ascii(content: &str, config: &MeshOptConfig) -> OptResult<String> {
    let mut lines: Vec<&str> = content.lines().collect();
    
    if config.weld_vertices {
        // Remove comment lines and empty lines
        lines.retain(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with(';')
        });
    }
    
    if config.weld_vertices {
        // Simple whitespace compression for ASCII FBX
        // Note: This is a simplified approach - real FBX optimization would need proper parsing
        lines = lines.into_iter().map(|line| {
            if line.contains("Vertices:") || line.contains("Normals:") {
                // Just return the original line for now - proper implementation would
                // parse and compress the vertex data
                line
            } else {
                line
            }
        }).collect();
    }
    
    Ok(lines.join("\n"))
}

/// Validate FBX file structure
pub fn validate_fbx_structure(data: &[u8]) -> OptResult<bool> {
    if data.len() < 23 {
        return Err(OptError::InvalidFormat("FBX file too small".to_string()));
    }
    
    // Check for binary FBX
    if data.starts_with(FBX_BINARY_SIGNATURE) {
        return Ok(true);
    }
    
    // Check for ASCII FBX
    if data.starts_with(FBX_ASCII_SIGNATURE) {
        let content = core::str::from_utf8(data)
            .map_err(|_| OptError::InvalidFormat("FBX ASCII contains invalid UTF-8".to_string()))?;
        
        // Basic validation for ASCII FBX
        if content.contains("FBX") && (content.contains("Objects:") || content.contains("Model:")) {
            return Ok(true);
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
            alloc::vec![0x00, 0x00], // Reserved
            alloc::vec![0xE8, 0x1C, 0x00, 0x00], // Version 7400
        ].concat();
        
        assert!(binary_fbx_header.starts_with(FBX_BINARY_SIGNATURE));
        
        let ascii_fbx = b"; FBX 7.4.0 project file";
        assert!(ascii_fbx.starts_with(FBX_ASCII_SIGNATURE));
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
