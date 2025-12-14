extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}, format};

use crate::types::{MeshOptConfig, OptResult, OptError};
use crate::optimizers::get_current_time_ms;

pub fn optimize_obj(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    optimize_obj_advanced_text(data, config)
}

fn optimize_obj_advanced_text(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("Invalid UTF-8 in OBJ file".to_string()))?;
    
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut texcoords = Vec::new();
    let mut faces = Vec::new();
    let mut object_name = String::from("optimized_mesh");
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        match parts[0] {
            "o" => {
                if parts.len() > 1 {
                    object_name = parts[1].to_string();
                }
            },
            "v" => {
                if parts.len() >= 4 {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>()
                    ) {
                        vertices.push(x);
                        vertices.push(y);
                        vertices.push(z);
                    }
                }
            },
            "vn" => {
                if parts.len() >= 4 {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>()
                    ) {
                        normals.push(x);
                        normals.push(y);
                        normals.push(z);
                    }
                }
            },
            "vt" => {
                if parts.len() >= 3 {
                    if let (Ok(u), Ok(v)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>()
                    ) {
                        texcoords.push(u);
                        texcoords.push(v);
                    }
                }
            },
            "f" => {
                let mut face_indices = Vec::new();
                for i in 1..parts.len() {
                    if let Some(vertex_idx) = parts[i].split('/').next() {
                        if let Ok(idx) = vertex_idx.parse::<u32>() {
                            if idx > 0 {
                                face_indices.push(idx - 1);
                            }
                        }
                    }
                }
                
                if face_indices.len() == 3 {
                    faces.extend_from_slice(&face_indices);
                } else if face_indices.len() == 4 {
                    faces.push(face_indices[0]);
                    faces.push(face_indices[1]);
                    faces.push(face_indices[2]);
                    
                    faces.push(face_indices[0]);
                    faces.push(face_indices[2]);
                    faces.push(face_indices[3]);
                }
            },
            _ => {}
        }
    }
    
    let _start_time = get_current_time_ms();
    #[cfg(c_hotspots_available)]
    let data_size = vertices.len() * 4 + faces.len() * 4;
    
    let (optimized_vertices, optimized_indices) = if config.target_ratio < 1.0 && !faces.is_empty() {
        #[cfg(c_hotspots_available)]
        {
            if data_size > 100_000 {
                apply_simple_decimation(&vertices, &faces, config.target_ratio, config)?
            } else {
                apply_simple_decimation(&vertices, &faces, config.target_ratio, config)?
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            apply_simple_decimation(&vertices, &faces, config.target_ratio, config)?
        }
    } else {
        (vertices, faces)
    };
    
    let (final_vertices, final_indices) = if config.weld_vertices {
        #[cfg(c_hotspots_available)]
        {
            if data_size > 50_000 {
                apply_vertex_welding(&optimized_vertices, &optimized_indices, config.vertex_tolerance)?
            } else {
                apply_vertex_welding(&optimized_vertices, &optimized_indices, config.vertex_tolerance)?
            }
        }
        
        #[cfg(not(c_hotspots_available))]
        {
            apply_vertex_welding(&optimized_vertices, &optimized_indices, config.vertex_tolerance)?
        }
    } else {
        (optimized_vertices, optimized_indices)
    };
    
    generate_optimized_obj_content(&object_name, &final_vertices, &final_indices, &texcoords, config)
}

fn apply_simple_decimation(
    vertices: &[f32], 
    indices: &[u32], 
    target_ratio: f32,
    config: &MeshOptConfig
) -> OptResult<(Vec<f32>, Vec<u32>)> {
    let target_triangle_count = ((indices.len() / 3) as f32 * target_ratio) as usize;
    
    if config.preserve_topology {
        let mut new_indices = Vec::new();
        
        for chunk in indices.chunks(3) {
            if new_indices.len() / 3 < target_triangle_count {
                new_indices.extend_from_slice(chunk);
            }
        }
        
        Ok((vertices.to_vec(), new_indices))
    } else {
        let mut new_indices = Vec::new();
        let step = (indices.len() / 3).max(1) / target_triangle_count.max(1);
        
        for i in (0..indices.len()).step_by(step * 3) {
            if i + 2 < indices.len() && new_indices.len() / 3 < target_triangle_count {
                new_indices.push(indices[i]);
                new_indices.push(indices[i + 1]);
                new_indices.push(indices[i + 2]);
            }
        }
        
        Ok((vertices.to_vec(), new_indices))
    }
}

fn apply_vertex_welding(
    vertices: &[f32], 
    indices: &[u32], 
    tolerance: f32
) -> OptResult<(Vec<f32>, Vec<u32>)> {
    use alloc::collections::BTreeMap;
    
    let mut vertex_map = BTreeMap::new();
    let mut new_vertices = Vec::new();
    let mut index_mapping = Vec::new();
    
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
            index_mapping.push(existing_index);
        } else {
            let new_index = new_vertices.len() / 3;
            vertex_map.insert(hash_key, new_index as u32);
            index_mapping.push(new_index as u32);
            
            new_vertices.push(x);
            new_vertices.push(y);
            new_vertices.push(z);
        }
    }
    
    let mut new_indices = Vec::new();
    for &idx in indices.iter() {
        if (idx as usize) < index_mapping.len() {
            new_indices.push(index_mapping[idx as usize]);
        }
    }
    
    Ok((new_vertices, new_indices))
}

fn generate_optimized_obj_content(
    object_name: &str,
    vertices: &[f32],
    indices: &[u32],
    texcoords: &[f32],
    config: &MeshOptConfig
) -> OptResult<Vec<u8>> {
    let mut content = String::new();
    content.push_str("# Optimized by Pixie Juice\n");
    content.push_str(&format!("o {}\n", object_name));
    
    for chunk in vertices.chunks(3) {
        if chunk.len() == 3 {
            content.push_str(&format!("v {} {} {}\n", chunk[0], chunk[1], chunk[2]));
        }
    }
    
    for chunk in texcoords.chunks(2) {
        if chunk.len() == 2 {
            content.push_str(&format!("vt {} {}\n", chunk[0], chunk[1]));
        }
    }
    
    if config.generate_normals {
        let normals = generate_normals_simple(vertices, indices);
        for chunk in normals.chunks(3) {
            if chunk.len() == 3 {
                content.push_str(&format!("vn {} {} {}\n", chunk[0], chunk[1], chunk[2]));
            }
        }
    }
    
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            content.push_str(&format!("f {} {} {}\n", 
                chunk[0] + 1, chunk[1] + 1, chunk[2] + 1));
        }
    }
    
    Ok(content.into_bytes())
}

fn generate_normals_simple(vertices: &[f32], indices: &[u32]) -> Vec<f32> {
    let mut normals = Vec::with_capacity(vertices.len());
    normals.resize(vertices.len(), 0.0);
    
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            let i0 = chunk[0] as usize * 3;
            let i1 = chunk[1] as usize * 3;
            let i2 = chunk[2] as usize * 3;
            
            if i0 + 2 < vertices.len() && i1 + 2 < vertices.len() && i2 + 2 < vertices.len() {
                let v0 = [vertices[i0], vertices[i0 + 1], vertices[i0 + 2]];
                let v1 = [vertices[i1], vertices[i1 + 1], vertices[i1 + 2]];
                let v2 = [vertices[i2], vertices[i2 + 1], vertices[i2 + 2]];
                
                let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
                let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
                
                let normal = [
                    edge1[1] * edge2[2] - edge1[2] * edge2[1],
                    edge1[2] * edge2[0] - edge1[0] * edge2[2],
                    edge1[0] * edge2[1] - edge1[1] * edge2[0],
                ];
                
                for &idx in chunk.iter() {
                    let ni = idx as usize * 3;
                    if ni + 2 < normals.len() {
                        normals[ni] += normal[0];
                        normals[ni + 1] += normal[1];
                        normals[ni + 2] += normal[2];
                    }
                }
            }
        }
    }
    
    for chunk in normals.chunks_mut(3) {
        if chunk.len() == 3 {
            let length = (chunk[0] * chunk[0] + chunk[1] * chunk[1] + chunk[2] * chunk[2]).sqrt();
            if length > 0.0001 {
                chunk[0] /= length;
                chunk[1] /= length;
                chunk[2] /= length;
            }
        }
    }
    
    normals
}

pub fn is_obj(data: &[u8]) -> bool {
    if let Ok(content) = core::str::from_utf8(data) {
        let lines: Vec<&str> = content.lines().take(10).collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("v ") || trimmed.starts_with("vn ") || 
               trimmed.starts_with("vt ") || trimmed.starts_with("f ") {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_obj_optimization() {
        let config = MeshOptConfig::default();
        let result = optimize_obj(&[], &config);
        assert!(result.is_err());
        let obj_content = b"# Simple OBJ file\nv 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3\n";
        let result = optimize_obj(obj_content, &config);
        assert!(result.is_ok());
    }
}
