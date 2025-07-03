//! glTF and GLB format support using gltf crate

use crate::types::{OptConfig, OptResult};
use crate::mesh::loader::{load_mesh_data, write_mesh_data};
use crate::ffi::mesh_ffi::{decimate_mesh_safe, weld_vertices_safe};

/// Optimize glTF JSON format
pub fn optimize_gltf(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load mesh data
    let mut mesh = load_mesh_data(data, "gltf")?;
    
    // Apply mesh optimization
    if let Some(target_reduction) = config.target_reduction {
        if target_reduction > 0.0 && target_reduction < 1.0 {
            mesh = decimate_mesh_safe(&mesh, 1.0 - target_reduction)?;
        }
    }
    
    // Apply vertex welding (with small tolerance)
    mesh = weld_vertices_safe(&mesh, 0.001)?;
    
    // For now, convert to OBJ format as output
    // TODO: Implement proper glTF writing
    write_mesh_data(&mesh, "obj")
}

/// Optimize GLB binary format  
pub fn optimize_glb(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load mesh data
    let mut mesh = load_mesh_data(data, "glb")?;
    
    // Apply mesh optimization
    if let Some(target_reduction) = config.target_reduction {
        if target_reduction > 0.0 && target_reduction < 1.0 {
            mesh = decimate_mesh_safe(&mesh, 1.0 - target_reduction)?;
        }
    }
    
    // Apply vertex welding (with small tolerance)
    mesh = weld_vertices_safe(&mesh, 0.001)?;
    
    // For now, convert to OBJ format as output
    // TODO: Implement proper GLB writing
    write_mesh_data(&mesh, "obj")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gltf_optimization() {
        let config = OptConfig::default();
        
        // Test with empty data should fail gracefully
        let result = optimize_gltf(&[], &config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_glb_optimization() {
        let config = OptConfig::default();
        
        // Test with empty data should fail gracefully
        let result = optimize_glb(&[], &config);
        assert!(result.is_err());
    }
}
