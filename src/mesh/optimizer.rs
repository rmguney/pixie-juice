extern crate alloc;
use alloc::{vec::Vec, string::ToString};
use crate::types::{PixieResult, PixieError, MeshOptConfig};
use crate::optimizers::{get_current_time_ms, update_performance_stats};

#[cfg(c_hotspots_available)]
use crate::c_hotspots;

pub struct MeshOptimizerCore;

impl MeshOptimizerCore {
    pub fn decimate_mesh_qem(
        vertices: &[f32], 
        indices: &[u32], 
        target_ratio: f32,
        config: &MeshOptConfig
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;
        
        #[cfg(c_hotspots_available)]
        {
            if data_size > 200_000 {
            }
        }
        
        let result = match config.simplification_algorithm {
            crate::types::SimplificationAlgorithm::QuadricErrorMetrics => {
                decimate_qem_rust(vertices, indices, target_ratio, config)
            },
            crate::types::SimplificationAlgorithm::EdgeCollapse => {
                decimate_edge_collapse_rust(vertices, indices, target_ratio, config)
            },
            crate::types::SimplificationAlgorithm::VertexClustering => {
                decimate_vertex_clustering_rust(vertices, indices, target_ratio, config)
            },
        };
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        match result {
            Ok(decimated) => Ok(decimated),
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn weld_vertices(
        vertices: &[f32], 
        indices: &[u32], 
        tolerance: f32
    ) -> PixieResult<(Vec<f32>, Vec<u32>)> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;
        
        let result = weld_vertices_spatial_hash(vertices, indices, tolerance);
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        result
    }

    pub fn optimize_vertex_cache(
        vertices: &[f32], 
        indices: &[u32]
    ) -> PixieResult<Vec<u32>> {
        let start_time = get_current_time_ms();
        let data_size = vertices.len() * 4 + indices.len() * 4;
        
        let result = optimize_vertex_cache_forsyth(indices);
        
        let elapsed = get_current_time_ms() - start_time;
        update_performance_stats(false, elapsed, data_size);
        
        result
    }
}

#[cfg(c_hotspots_available)]
fn apply_c_mesh_decimation(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    use crate::c_hotspots::decimate_mesh_qem;
    
    unsafe {
        let result = decimate_mesh_qem(
            vertices.as_ptr(),
            vertices.len(),
            indices.as_ptr(),
            indices.len(),
            target_ratio
        );
        
        if result.success != 0 {
            let new_vertices = Vec::from_raw_parts(
                result.vertices,
                result.vertex_count,
                result.vertex_count
            );
            let new_indices = Vec::from_raw_parts(
                result.indices,
                result.index_count,
                result.index_count
            );
            Ok((new_vertices, new_indices))
        } else {
            Err(PixieError::CHotspotError("Mesh decimation failed".to_string()))
        }
    }
}

fn decimate_qem_rust(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32,
    config: &MeshOptConfig
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    if vertices.len() % 3 != 0 {
        return Err(PixieError::MeshOptimizationFailed(
            "Invalid vertex data: must be multiples of 3".to_string()
        ));
    }
    
    if indices.len() % 3 != 0 {
        return Err(PixieError::MeshOptimizationFailed(
            "Invalid index data: must be multiples of 3".to_string()
        ));
    }
    
    let target_triangle_count = ((indices.len() / 3) as f32 * target_ratio) as usize;
    
    if config.preserve_topology {
        let _decimation_step = if target_ratio > 0.5 { 2 } else { 3 };
        let mut new_indices = Vec::new();
        
        for chunk in indices.chunks(3) {
            if new_indices.len() / 3 < target_triangle_count {
                new_indices.extend_from_slice(chunk);
            }
        }
        
        Ok((vertices.to_vec(), new_indices))
    } else {
        let mut new_indices = Vec::new();
        let step = indices.len() / 3 / target_triangle_count.max(1);
        
        for i in (0..indices.len()).step_by(step * 3) {
            if i + 2 < indices.len() {
                new_indices.push(indices[i]);
                new_indices.push(indices[i + 1]);
                new_indices.push(indices[i + 2]);
            }
        }
        
        Ok((vertices.to_vec(), new_indices))
    }
}

fn decimate_edge_collapse_rust(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32,
    _config: &MeshOptConfig
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    let target_count = ((indices.len() / 3) as f32 * target_ratio) as usize * 3;
    let mut new_indices = indices.to_vec();
    new_indices.truncate(target_count);
    
    Ok((vertices.to_vec(), new_indices))
}

fn decimate_vertex_clustering_rust(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32,
    _config: &MeshOptConfig
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    let target_count = ((indices.len() / 3) as f32 * target_ratio) as usize * 3;
    let mut new_indices = indices.to_vec();
    new_indices.truncate(target_count);
    
    Ok((vertices.to_vec(), new_indices))
}

fn weld_vertices_spatial_hash(
    vertices: &[f32], 
    indices: &[u32], 
    tolerance: f32
) -> PixieResult<(Vec<f32>, Vec<u32>)> {
    use alloc::collections::BTreeMap;
    
    let mut vertex_map = BTreeMap::new();
    let mut new_vertices = Vec::new();
    let mut new_indices = Vec::new();
    
    let inv_tolerance = 1.0 / tolerance;
    
    for i in 0..vertices.len() / 3 {
        let x = vertices[i * 3];
        let y = vertices[i * 3 + 1];
        let z = vertices[i * 3 + 2];
        
        let hash_x = (x * inv_tolerance) as i32;
        let hash_y = (y * inv_tolerance) as i32;
        let hash_z = (z * inv_tolerance) as i32;
        let hash_key = (hash_x, hash_y, hash_z);
        
        if let Some(&existing_index) = vertex_map.get(&hash_key) {
            for &index in indices.iter() {
                if index == i as u32 {
                    new_indices.push(existing_index);
                } else {
                    new_indices.push(index);
                }
            }
        } else {
            let new_index = new_vertices.len() as u32 / 3;
            vertex_map.insert(hash_key, new_index);
            new_vertices.push(x);
            new_vertices.push(y);
            new_vertices.push(z);
        }
    }
    
    Ok((new_vertices, new_indices))
}

fn optimize_vertex_cache_forsyth(indices: &[u32]) -> PixieResult<Vec<u32>> {
    
    let mut optimized = indices.to_vec();
    let _triangle_count = indices.len() / 3;

    optimized.sort_by(|a, b| {
        let tri_a = a / 3;
        let tri_b = b / 3;
        tri_a.cmp(&tri_b)
    });
    
    Ok(optimized)
}
