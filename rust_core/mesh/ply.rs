//! PLY format support using ply-rs crate

use crate::types::{OptConfig, OptResult};
use crate::mesh::loader::{load_mesh_data, write_mesh_data};
use crate::ffi::mesh_ffi::{decimate_mesh_safe, weld_vertices_safe};

/// Optimize PLY format
pub fn optimize_ply(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load mesh data
    let mut mesh = load_mesh_data(data, "ply")?;
    
    // Apply mesh optimization
    if let Some(target_reduction) = config.target_reduction {
        if target_reduction > 0.0 && target_reduction < 1.0 {
            mesh = decimate_mesh_safe(&mesh, 1.0 - target_reduction)?;
        }
    }
    
    // Apply vertex welding (with small tolerance)
    mesh = weld_vertices_safe(&mesh, 0.001)?;
    
    // Write back to PLY format
    write_mesh_data(&mesh, "ply")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ply_optimization() {
        let config = OptConfig::default();
        
        // Test with empty data should fail gracefully
        let result = optimize_ply(&[], &config);
        assert!(result.is_err());
    }
}
