//! OBJ format support using obj crate

use crate::types::{OptConfig, OptError, OptResult};
use crate::mesh::loader::{load_mesh_data, write_mesh_data};
use crate::ffi::mesh_ffi::{decimate_mesh_safe, weld_vertices_safe};

/// Optimize OBJ format
pub fn optimize_obj(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load mesh data
    let mut mesh = load_mesh_data(data, "obj")?;
    
    // Apply mesh optimization
    if let Some(target_reduction) = config.target_reduction {
        if target_reduction > 0.0 && target_reduction < 1.0 {
            mesh = decimate_mesh_safe(&mesh, 1.0 - target_reduction)?;
        }
    }
    
    // Apply vertex welding (with small tolerance)
    mesh = weld_vertices_safe(&mesh, 0.001)?;
    
    // Write back to OBJ format
    write_mesh_data(&mesh, "obj")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_obj_optimization() {
        let config = OptConfig::default();
        
        // Test with empty data should fail gracefully
        let result = optimize_obj(&[], &config);
        assert!(result.is_err());
        
        // Test with minimal OBJ content
        let obj_content = b"# Simple OBJ file\nv 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3\n";
        let result = optimize_obj(obj_content, &config);
        assert!(result.is_ok());
    }
}
