//! glTF and GLB format support using gltf crate

use crate::types::{OptConfig, OptResult, OptError};
use crate::mesh::loader::load_mesh_data;
use crate::ffi::mesh_ffi::{decimate_mesh_safe, weld_vertices_safe, MeshData};
use serde_json::json;

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
    
    // Write optimized mesh as proper glTF
    write_gltf_json(&mesh)
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
    
    // Convert optimized mesh to glTF JSON for now
    // TODO: Implement proper GLB binary writing
    write_gltf_json(&mesh)
}

/// Write mesh data as glTF JSON format
fn write_gltf_json(mesh: &MeshData) -> OptResult<Vec<u8>> {
    // Create a minimal glTF structure
    let gltf_doc = json!({
        "asset": {
            "version": "2.0",
            "generator": "Pixie Juice Optimizer"
        },
        "scene": 0,
        "scenes": [
            {
                "nodes": [0]
            }
        ],
        "nodes": [
            {
                "mesh": 0
            }
        ],
        "meshes": [
            {
                "primitives": [
                    {
                        "attributes": {
                            "POSITION": 0
                        },
                        "indices": 1
                    }
                ]
            }
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126, // FLOAT
                "count": mesh.vertices.len() / 3,
                "type": "VEC3",
                "min": calculate_min_position(&mesh.vertices),
                "max": calculate_max_position(&mesh.vertices)
            },
            {
                "bufferView": 1,
                "componentType": 5123, // UNSIGNED_SHORT (assuming indices fit in u16)
                "count": mesh.indices.len(),
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": mesh.vertices.len() * 4
            },
            {
                "buffer": 0,
                "byteOffset": mesh.vertices.len() * 4,
                "byteLength": mesh.indices.len() * 2
            }
        ],
        "buffers": [
            {
                "byteLength": mesh.vertices.len() * 4 + mesh.indices.len() * 2
            }
        ]
    });
    
    serde_json::to_vec_pretty(&gltf_doc)
        .map_err(|e| OptError::ProcessingError(format!("Failed to serialize glTF: {}", e)))
}

/// Calculate min position for glTF accessor
fn calculate_min_position(vertices: &[f32]) -> Vec<f32> {
    if vertices.len() < 3 {
        return vec![0.0, 0.0, 0.0];
    }
    
    let mut min = [vertices[0], vertices[1], vertices[2]];
    for chunk in vertices.chunks(3) {
        if chunk.len() == 3 {
            min[0] = min[0].min(chunk[0]);
            min[1] = min[1].min(chunk[1]);
            min[2] = min[2].min(chunk[2]);
        }
    }
    min.to_vec()
}

/// Calculate max position for glTF accessor
fn calculate_max_position(vertices: &[f32]) -> Vec<f32> {
    if vertices.len() < 3 {
        return vec![0.0, 0.0, 0.0];
    }
    
    let mut max = [vertices[0], vertices[1], vertices[2]];
    for chunk in vertices.chunks(3) {
        if chunk.len() == 3 {
            max[0] = max[0].max(chunk[0]);
            max[1] = max[1].max(chunk[1]);
            max[2] = max[2].max(chunk[2]);
        }
    }
    max.to_vec()
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
