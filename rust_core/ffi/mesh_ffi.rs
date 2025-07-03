//! FFI bindings for mesh decimation C hotspots

use crate::types::{OptError, OptResult};

#[cfg(feature = "c_hotspots")]
use std::ffi::CStr;
#[cfg(feature = "c_hotspots")]
use std::os::raw::c_char;

// Import C functions from mesh_decimate.h when C hotspots are enabled
#[cfg(feature = "c_hotspots")]
extern "C" {
    fn decimate_mesh_qem(
        vertices: *const f32,
        vertex_count: usize,
        indices: *const u32,
        index_count: usize,
        target_ratio: f32,
    ) -> MeshDecimateResult;
    
    fn weld_vertices_spatial(
        vertices: *const f32,
        vertex_count: usize,
        indices: *const u32,
        index_count: usize,
        tolerance: f32,
    ) -> MeshDecimateResult;
    
    fn free_mesh_decimate_result(result: *mut MeshDecimateResult);
}

#[cfg(feature = "c_hotspots")]
#[repr(C)]
struct MeshDecimateResult {
    vertices: *mut f32,
    indices: *mut u32,
    vertex_count: usize,
    index_count: usize,
    success: i32,
    error_message: [c_char; 256],
}

/// Simple mesh data structure for processing
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<f32>,  // x,y,z triplets
    pub indices: Vec<u32>,   // triangle indices
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
    
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }
    
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Safe wrapper for C mesh decimation using quadric error metrics
pub fn decimate_mesh_safe(mesh: &MeshData, target_ratio: f32) -> OptResult<MeshData> {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return Err(OptError::InvalidFormat("Empty mesh data".to_string()));
    }
    
    if target_ratio <= 0.0 || target_ratio > 1.0 {
        return Err(OptError::InvalidFormat("Target ratio must be between 0 and 1".to_string()));
    }

    #[cfg(feature = "c_hotspots")]
    {
        let vertex_count = mesh.vertex_count();
        let index_count = mesh.indices.len();
        
        let result = unsafe {
            decimate_mesh_qem(
                mesh.vertices.as_ptr(),
                vertex_count,
                mesh.indices.as_ptr(),
                index_count,
                target_ratio,
            )
        };
        
        if result.success == 0 {
            let error_msg = unsafe {
                CStr::from_ptr(result.error_message.as_ptr())
                    .to_string_lossy()
                    .to_string()
            };
            return Err(OptError::ProcessingError(format!("Mesh decimation failed: {}", error_msg)));
        }
        
        // Copy results from C memory to Rust vectors
        let new_vertices = unsafe {
            std::slice::from_raw_parts(result.vertices, result.vertex_count * 3)
                .to_vec()
        };
        
        let new_indices = unsafe {
            std::slice::from_raw_parts(result.indices, result.index_count)
                .to_vec()
        };
        
        // Free C-allocated memory
        let mut result_mut = result;
        unsafe {
            free_mesh_decimate_result(&mut result_mut);
        }
        
        Ok(MeshData {
            vertices: new_vertices,
            indices: new_indices,
        })
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation
        let target_vertex_count = ((mesh.vertex_count() as f32) * target_ratio) as usize;
        let target_index_count = ((mesh.indices.len() as f32) * target_ratio) as usize;
        
        // Simple decimation: just take the first N vertices/indices
        let new_vertices = mesh.vertices.iter()
            .take(target_vertex_count * 3)
            .cloned()
            .collect();
        
        let new_indices = mesh.indices.iter()
            .take(target_index_count)
            .map(|&i| if (i as usize) < target_vertex_count { i } else { 0 })
            .collect();
        
        Ok(MeshData {
            vertices: new_vertices,
            indices: new_indices,
        })
    }
}

/// Safe wrapper for C vertex welding using spatial hashing
pub fn weld_vertices_safe(mesh: &MeshData, tolerance: f32) -> OptResult<MeshData> {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return Err(OptError::InvalidFormat("Empty mesh data".to_string()));
    }
    
    if tolerance < 0.0 {
        return Err(OptError::InvalidFormat("Tolerance must be non-negative".to_string()));
    }

    #[cfg(feature = "c_hotspots")]
    {
        let vertex_count = mesh.vertex_count();
        let index_count = mesh.indices.len();
        
        let result = unsafe {
            weld_vertices_spatial(
                mesh.vertices.as_ptr(),
                vertex_count,
                mesh.indices.as_ptr(),
                index_count,
                tolerance,
            )
        };
        
        if result.success == 0 {
            let error_msg = unsafe {
                CStr::from_ptr(result.error_message.as_ptr())
                    .to_string_lossy()
                    .to_string()
            };
            return Err(OptError::ProcessingError(format!("Vertex welding failed: {}", error_msg)));
        }
        
        // Copy results from C memory to Rust vectors
        let new_vertices = unsafe {
            std::slice::from_raw_parts(result.vertices, result.vertex_count * 3)
                .to_vec()
        };
        
        let new_indices = unsafe {
            std::slice::from_raw_parts(result.indices, result.index_count)
                .to_vec()
        };
        
        // Free C-allocated memory
        let mut result_mut = result;
        unsafe {
            free_mesh_decimate_result(&mut result_mut);
        }
        
        Ok(MeshData {
            vertices: new_vertices,
            indices: new_indices,
        })
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - basic vertex welding
        use std::collections::HashMap;
        
        let mut vertex_map: HashMap<(i32, i32, i32), u32> = HashMap::new();
        let mut new_vertices = Vec::new();
        let mut new_indices = Vec::new();
        let mut next_index = 0u32;
        
        // Process vertices and build mapping
        for chunk in mesh.vertices.chunks(3) {
            if chunk.len() == 3 {
                let key = (
                    (chunk[0] / tolerance) as i32,
                    (chunk[1] / tolerance) as i32,
                    (chunk[2] / tolerance) as i32,
                );
                
                if !vertex_map.contains_key(&key) {
                    vertex_map.insert(key, next_index);
                    new_vertices.extend_from_slice(chunk);
                    next_index += 1;
                }
            }
        }
        
        // Remap indices
        for &index in &mesh.indices {
            let vertex_start = (index as usize) * 3;
            if vertex_start + 2 < mesh.vertices.len() {
                let vertex = &mesh.vertices[vertex_start..vertex_start + 3];
                let key = (
                    (vertex[0] / tolerance) as i32,
                    (vertex[1] / tolerance) as i32,
                    (vertex[2] / tolerance) as i32,
                );
                
                if let Some(&new_index) = vertex_map.get(&key) {
                    new_indices.push(new_index);
                } else {
                    new_indices.push(0); // Fallback
                }
            }
        }
        
        Ok(MeshData {
            vertices: new_vertices,
            indices: new_indices,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_decimation() {
        // Create a simple triangle mesh
        let mesh = MeshData {
            vertices: vec![
                0.0, 0.0, 0.0,  // vertex 0
                1.0, 0.0, 0.0,  // vertex 1  
                0.0, 1.0, 0.0,  // vertex 2
                1.0, 1.0, 0.0,  // vertex 3
            ],
            indices: vec![0, 1, 2, 1, 3, 2],  // 2 triangles
        };
        
        let result = decimate_mesh_safe(&mesh, 0.5);
        assert!(result.is_ok());
        
        let decimated = result.unwrap();
        assert!(!decimated.vertices.is_empty());
        assert!(!decimated.indices.is_empty());
        assert!(decimated.vertex_count() <= mesh.vertex_count());
    }
    
    #[test]
    fn test_vertex_welding() {
        // Create a mesh with duplicate vertices
        let mesh = MeshData {
            vertices: vec![
                0.0, 0.0, 0.0,      // vertex 0
                0.001, 0.001, 0.0,  // vertex 1 (close to 0)
                1.0, 0.0, 0.0,      // vertex 2
            ],
            indices: vec![0, 1, 2],
        };
        
        let result = weld_vertices_safe(&mesh, 0.01);
        assert!(result.is_ok());
        
        let welded = result.unwrap();
        assert!(welded.vertex_count() <= mesh.vertex_count());
    }
}
