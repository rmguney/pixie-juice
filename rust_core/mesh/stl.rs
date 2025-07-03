//! STL format support using stl_io crate

use crate::types::{OptConfig, OptResult};
use crate::mesh::loader::{load_mesh_data, write_mesh_data};
use crate::ffi::mesh_ffi::{decimate_mesh_safe, weld_vertices_safe};

/// Optimize STL format (both ASCII and binary)
pub fn optimize_stl(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load mesh data
    let mut mesh = load_mesh_data(data, "stl")?;
    
    // Apply mesh optimization
    if let Some(target_reduction) = config.target_reduction {
        if target_reduction > 0.0 && target_reduction < 1.0 {
            mesh = decimate_mesh_safe(&mesh, 1.0 - target_reduction)?;
        }
    }
    
    // Apply vertex welding (with small tolerance)
    mesh = weld_vertices_safe(&mesh, 0.001)?;
    
    // Write back to STL format
    write_mesh_data(&mesh, "stl")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stl_optimization() {
        let config = OptConfig::default();
        
        // Test with empty data should fail gracefully
        let result = optimize_stl(&[], &config);
        assert!(result.is_err());
        
        // Test with minimal ASCII STL content
        let stl_content = b"solid test\nfacet normal 0.0 0.0 1.0\nouter loop\nvertex 0.0 0.0 0.0\nvertex 1.0 0.0 0.0\nvertex 0.0 1.0 0.0\nendloop\nendfacet\nendsolid test\n";
        let result = optimize_stl(stl_content, &config);
        assert!(result.is_ok());
    }
}
