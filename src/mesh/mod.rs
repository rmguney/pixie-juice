//! Mesh processing and optimization module

extern crate alloc;
use alloc::{vec::Vec, string::ToString};

use crate::types::{OptError, OptResult, MeshOptConfig};
use crate::formats::MeshFormat;

// Core mesh format modules
pub mod obj;        // Wavefront OBJ - via tobj crate
pub mod gltf;       // glTF 2.0 - via gltf crate  
pub mod ply;        // Stanford PLY - via ply-rs crate or custom parser
pub mod stl;        // STL - via stl_io crate
pub mod fbx;        // Autodesk FBX - custom binary parser

// Support modules
pub mod formats;    // Format detection and metadata
pub mod loader;     // Mesh loading utilities
pub mod optimizer;  // Core optimization algorithms
pub mod validator;  // Mesh validation

/// Mesh optimizer that delegates to format-specific optimizers
#[derive(Debug, Clone)]
pub struct MeshOptimizer {
    config: MeshOptConfig,
}

impl MeshOptimizer {
    /// Create a new mesh optimizer with the given configuration
    pub fn new(config: MeshOptConfig) -> Self {
        Self { config }
    }

    /// Optimize a mesh based on its detected format
    pub fn optimize(&self, data: &[u8]) -> OptResult<Vec<u8>> {
        let format = detect_mesh_format(data)?;
        
        match format {
            MeshFormat::Obj => obj::optimize_obj(data, &self.config),
            MeshFormat::Gltf => gltf::optimize_gltf(data, &self.config),
            MeshFormat::Glb => gltf::optimize_glb(data, &self.config),
            MeshFormat::Ply => ply::optimize_ply(data, &self.config),
            MeshFormat::Stl => stl::optimize_stl(data, &self.config),
            MeshFormat::Fbx => fbx::optimize_fbx(data, &self.config),
        }
    }

    /// Analyze mesh format and basic properties
    pub fn analyze(&self, data: &[u8]) -> OptResult<crate::types::MeshInfo> {
        let _format = detect_mesh_format(data)?;
        // TODO: Return actual mesh analysis
        Ok(crate::types::MeshInfo::default())
    }
}

impl Default for MeshOptimizer {
    fn default() -> Self {
        Self::new(MeshOptConfig::default())
    }
}

/// Detect mesh format from file signature
pub fn detect_mesh_format(data: &[u8]) -> OptResult<MeshFormat> {
    if data.len() < 8 {
        return Err(OptError::InvalidFormat("File too small".to_string()));
    }

    // GLTF/GLB signature
    if data.starts_with(b"glTF") {
        return Ok(MeshFormat::Glb);
    }

    // Check for JSON-based GLTF
    if data.starts_with(b"{") && data.len() > 20 {
        // Look for GLTF-specific JSON properties
        let start = core::str::from_utf8(&data[..data.len().min(1024)]).ok();
        if let Some(text) = start {
            if text.contains("\"asset\"") && text.contains("\"version\"") {
                return Ok(MeshFormat::Gltf);
            }
        }
    }
    
    // STL signature (binary)
    if data.len() >= 80 && data[0..5] == [0u8; 5] {
        // Check if it's a binary STL (80-byte header, then triangle count)
        if data.len() >= 84 {
            let triangle_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]);
            let expected_size = 84 + (triangle_count as usize * 50);
            if data.len() == expected_size {
                return Ok(MeshFormat::Stl);
            }
        }
    }

    // STL signature (ASCII)
    if data.starts_with(b"solid ") {
        return Ok(MeshFormat::Stl);
    }

    // PLY signature
    if data.starts_with(b"ply\n") || data.starts_with(b"ply\r\n") {
        return Ok(MeshFormat::Ply);
    }

    // FBX signature
    if data.starts_with(b"Kaydara FBX Binary") || data.starts_with(b"Autodesk FBX") {
        return Ok(MeshFormat::Fbx);
    }

    // FBX ASCII signature
    if data.starts_with(b"; FBX ") {
        return Ok(MeshFormat::Fbx);
    }

    // OBJ (heuristic - look for common OBJ keywords)
    if data.len() > 10 {
        let start = core::str::from_utf8(&data[..data.len().min(512)]).ok();
        if let Some(text) = start {
            if text.contains("v ") || text.contains("vn ") || text.contains("vt ") || text.contains("f ") {
                return Ok(MeshFormat::Obj);
            }
        }
    }

    Err(OptError::InvalidFormat("Unknown mesh format".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::MeshFormat;
    
    #[test]
    fn test_all_mesh_formats_accessible() {
        // Test that all mesh formats are properly accessible and can be detected
        let test_cases = [
            (b"v 1.0 1.0 1.0\nf 1 2 3".as_slice(), MeshFormat::Obj),
            (b"solid test\nfacet normal 0.0 0.0 1.0\n outer loop\n  vertex 0.0 0.0 0.0\nendloop\nendfacet\nendsolid test".as_slice(), MeshFormat::Stl),
            (b"ply\nformat ascii 1.0\nelement vertex 3\nend_header\n".as_slice(), MeshFormat::Ply),
            (b"; FBX 7.4.0 project file\nFBXHeaderExtension: {\n}\n".as_slice(), MeshFormat::Fbx),
            (b"{\"asset\": {\"version\": \"2.0\"}}".as_slice(), MeshFormat::Gltf),
            (b"glTF\x02\x00\x00\x00".as_slice(), MeshFormat::Glb),
        ];
        
        for (data, expected_format) in test_cases {
            match detect_mesh_format(data) {
                Ok(format) => {
                    assert_eq!(format, expected_format, "Format detection mismatch for {:?}", expected_format);
                    
                    // Test that the optimizer can handle this format
                    let optimizer = MeshOptimizer::default();
                    let _result = optimizer.optimize(data); // May succeed or fail, but should not panic
                },
                Err(e) => panic!("Failed to detect format {:?}: {:?}", expected_format, e),
            }
        }
    }
}