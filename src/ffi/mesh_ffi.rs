/// FFI bindings for mesh processing C hotspots
/// Full implementations with C hotspots and Rust fallbacks

use crate::types::{OptResult, OptError};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub vertex_count: usize,
    pub index_count: usize,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            vertex_count: 0,
            index_count: 0,
        }
    }
    
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    
    pub fn triangle_count(&self) -> usize {
        self.index_count / 3
    }
}

impl Default for MeshData {
    fn default() -> Self {
        Self::new()
    }
}

// C FFI declarations for mesh processing
#[cfg(c_hotspots_available)]
#[repr(C)]
#[derive(Debug)]
struct MeshDecimateResult {
    vertices: *mut f32,
    indices: *mut u32,
    vertex_count: usize,
    index_count: usize,
    success: i32,
    error_message: [u8; 256],
}

#[cfg(c_hotspots_available)]
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

/// Wrapper function to call C decimation with proper error handling
pub fn decimate_mesh_c(
    vertices: &[f32],
    indices: &[u32],
    target_ratio: f32,
) -> OptResult<(Vec<f32>, Vec<u32>)> {
    #[cfg(c_hotspots_available)]
    {
        if vertices.len() % 3 != 0 {
            return Err(OptError::ProcessingError("Vertex data must be multiple of 3 (x,y,z)".to_string()));
        }
        
        let vertex_count = vertices.len() / 3;
        
        unsafe {
            let result = decimate_mesh_qem(
                vertices.as_ptr(),
                vertex_count,
                indices.as_ptr(),
                indices.len(),
                target_ratio,
            );
            
            if result.success == 0 {
                return Err(OptError::ProcessingError("C mesh decimation failed".to_string()));
            }
            
            // Convert C result to Rust vectors
            let new_vertices = std::slice::from_raw_parts(result.vertices, result.vertex_count * 3).to_vec();
            let new_indices = std::slice::from_raw_parts(result.indices, result.index_count).to_vec();
            
            // Free C memory
            let mut result_copy = result;
            free_mesh_decimate_result(&mut result_copy);
            
            Ok((new_vertices, new_indices))
        }
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback implementation
        let _ = (vertices, indices, target_ratio);
        Err(OptError::ProcessingError("C hotspots not available".to_string()))
    }
}

/// Safe wrapper for mesh decimation
pub fn decimate_mesh_safe(
    mesh: &MeshData,
    target_ratio: f32,
) -> OptResult<MeshData> {
    if mesh.vertices.is_empty() || mesh.indices.is_empty() {
        return Err(OptError::ProcessingError("Empty mesh data".to_string()));
    }
    
    if target_ratio <= 0.0 || target_ratio > 1.0 {
        return Err(OptError::ProcessingError("Target ratio must be between 0 and 1".to_string()));
    }

    #[cfg(c_hotspots_available)]
    {
        // Use C hotspot implementation
        unsafe {
            let result = decimate_mesh_qem(
                mesh.vertices.as_ptr(),
                mesh.vertices.len() / 3, // vertex count
                mesh.indices.as_ptr(),
                mesh.indices.len(),
                target_ratio
            );
            
            if result.success == 0 {
                let error_msg = std::ffi::CStr::from_ptr(result.error_message.as_ptr() as *const i8)
                    .to_string_lossy()
                    .to_string();
                return Err(OptError::ProcessingError(format!("C mesh decimation failed: {}", error_msg)));
            }
            
            // Convert C result to Rust MeshData
            let vertices = std::slice::from_raw_parts(result.vertices, result.vertex_count * 3).to_vec();
            let indices = std::slice::from_raw_parts(result.indices, result.index_count).to_vec();
            
            let mut decimated_mesh = MeshData::new();
            decimated_mesh.vertices = vertices;
            decimated_mesh.indices = indices;
            decimated_mesh.vertex_count = result.vertex_count;
            decimated_mesh.index_count = result.index_count;
            
            // Free C memory
            let mut result_copy = result;
            free_mesh_decimate_result(&mut result_copy);
            
            Ok(decimated_mesh)
        }
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback implementation - simplified mesh decimation
        decimate_mesh_rust_fallback(mesh, target_ratio)
    }
}

/// Safe wrapper for vertex welding
pub fn weld_vertices_safe(
    mesh: &MeshData,
    tolerance: f32,
) -> OptResult<MeshData> {
    if mesh.vertices.is_empty() {
        return Err(OptError::ProcessingError("Empty mesh data".to_string()));
    }
    
    if tolerance < 0.0 {
        return Err(OptError::ProcessingError("Tolerance must be non-negative".to_string()));
    }

    #[cfg(c_hotspots_available)]
    {
        // Use C hotspot implementation
        unsafe {
            let result = weld_vertices_spatial(
                mesh.vertices.as_ptr(),
                mesh.vertices.len() / 3, // vertex count
                mesh.indices.as_ptr(),
                mesh.indices.len(),
                tolerance
            );
            
            if result.success == 0 {
                let error_msg = std::ffi::CStr::from_ptr(result.error_message.as_ptr() as *const i8)
                    .to_string_lossy()
                    .to_string();
                return Err(OptError::ProcessingError(format!("C vertex welding failed: {}", error_msg)));
            }
            
            // Convert C result to Rust MeshData
            let vertices = std::slice::from_raw_parts(result.vertices, result.vertex_count * 3).to_vec();
            let indices = std::slice::from_raw_parts(result.indices, result.index_count).to_vec();
            
            let mut welded_mesh = MeshData::new();
            welded_mesh.vertices = vertices;
            welded_mesh.indices = indices;
            welded_mesh.vertex_count = result.vertex_count;
            welded_mesh.index_count = result.index_count;
            
            // Free C memory
            let mut result_copy = result;
            free_mesh_decimate_result(&mut result_copy);
            
            Ok(welded_mesh)
        }
    }
    
    #[cfg(not(c_hotspots_available))]
    {
        // Rust fallback implementation
        weld_vertices_rust_fallback(mesh, tolerance)
    }
}

/// Rust fallback implementation for mesh decimation
#[cfg(not(c_hotspots_available))]
fn decimate_mesh_rust_fallback(mesh: &MeshData, target_ratio: f32) -> OptResult<MeshData> {
    // Simple decimation algorithm - remove every nth vertex based on target ratio
    let current_vertex_count = mesh.vertices.len() / 3;
    let target_vertex_count = ((current_vertex_count as f32) * target_ratio) as usize;
    
    if target_vertex_count >= current_vertex_count {
        // No decimation needed
        return Ok(mesh.clone());
    }
    
    let skip_factor = current_vertex_count / target_vertex_count.max(1);
    let mut new_vertices = Vec::new();
    let mut new_indices = Vec::new();
    let mut vertex_map = std::collections::HashMap::new();
    let mut new_vertex_index = 0u32;
    
    // Decimate vertices
    for i in (0..current_vertex_count).step_by(skip_factor.max(1)) {
        let base_idx = i * 3;
        if base_idx + 2 < mesh.vertices.len() {
            new_vertices.extend_from_slice(&mesh.vertices[base_idx..base_idx + 3]);
            vertex_map.insert(i as u32, new_vertex_index);
            new_vertex_index += 1;
        }
    }
    
    // Update indices
    for chunk in mesh.indices.chunks(3) {
        if chunk.len() == 3 {
            let mut valid_triangle = true;
            let mut new_triangle = [0u32; 3];
            
            for (j, &old_idx) in chunk.iter().enumerate() {
                if let Some(&new_idx) = vertex_map.get(&old_idx) {
                    new_triangle[j] = new_idx;
                } else {
                    valid_triangle = false;
                    break;
                }
            }
            
            if valid_triangle {
                new_indices.extend_from_slice(&new_triangle);
            }
        }
    }
    
    let mut result = MeshData::new();
    result.vertices = new_vertices;
    result.indices = new_indices;
    result.vertex_count = result.vertices.len() / 3;
    result.index_count = result.indices.len();
    
    Ok(result)
}

/// Rust fallback implementation for vertex welding
#[cfg(not(c_hotspots_available))]
fn weld_vertices_rust_fallback(mesh: &MeshData, tolerance: f32) -> OptResult<MeshData> {
    let vertex_count = mesh.vertices.len() / 3;
    let tolerance_sq = tolerance * tolerance;
    
    let mut unique_vertices = Vec::new();
    let mut vertex_map = Vec::new();
    
    // Find unique vertices within tolerance
    for i in 0..vertex_count {
        let base_idx = i * 3;
        let vertex = [
            mesh.vertices[base_idx],
            mesh.vertices[base_idx + 1],
            mesh.vertices[base_idx + 2],
        ];
        
        let mut found_match = None;
        
        // Check against existing unique vertices
        for (j, unique_vertex) in unique_vertices.chunks(3).enumerate() {
            let dx = vertex[0] - unique_vertex[0];
            let dy = vertex[1] - unique_vertex[1];
            let dz = vertex[2] - unique_vertex[2];
            let distance_sq = dx * dx + dy * dy + dz * dz;
            
            if distance_sq <= tolerance_sq {
                found_match = Some(j as u32);
                break;
            }
        }
        
        if let Some(existing_idx) = found_match {
            vertex_map.push(existing_idx);
        } else {
            let new_idx = unique_vertices.len() / 3;
            unique_vertices.extend_from_slice(&vertex);
            vertex_map.push(new_idx as u32);
        }
    }
    
    // Remap indices
    let mut new_indices = Vec::new();
    for &old_idx in &mesh.indices {
        if (old_idx as usize) < vertex_map.len() {
            new_indices.push(vertex_map[old_idx as usize]);
        }
    }
    
    let mut result = MeshData::new();
    result.vertices = unique_vertices;
    result.indices = new_indices;
    result.vertex_count = result.vertices.len() / 3;
    result.index_count = result.indices.len();
    
    Ok(result)
}
