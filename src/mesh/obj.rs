//! OBJ format support using obj crate

extern crate alloc;
use alloc::vec::Vec;

use crate::types::{MeshOptConfig, OptResult};

/// Optimize OBJ format
pub fn optimize_obj(data: &[u8], _config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    // Simple OBJ optimization: remove comments and empty lines
    let content = core::str::from_utf8(data)
        .map_err(|_| crate::types::OptError::InvalidFormat("Invalid UTF-8 in OBJ file".into()))?;
    
    let optimized_lines: Vec<&str> = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("mtllib")
        })
        .collect();
    
    let optimized_content = optimized_lines.join("\n");
    Ok(optimized_content.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_obj_optimization() {
        let config = MeshOptConfig::default();
        
        // Test with empty data should fail gracefully
        let result = optimize_obj(&[], &config);
        assert!(result.is_err());
        
        // Test with minimal OBJ content
        let obj_content = b"# Simple OBJ file\nv 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3\n";
        let result = optimize_obj(obj_content, &config);
        assert!(result.is_ok());
    }
}
