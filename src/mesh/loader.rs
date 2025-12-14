extern crate alloc;
use alloc::{vec::Vec, string::ToString, vec};
use crate::types::{OptResult, OptError, MeshOptConfig};
use crate::formats::MeshFormat;
use crate::mesh::formats::{detect_mesh_format, validate_mesh_data};

#[derive(Debug, Clone)]
pub struct MeshLoadResult {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub normals: Option<Vec<f32>>,
    pub uvs: Option<Vec<f32>>,
    pub format: MeshFormat,
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub metadata: MeshMetadata,
}

#[derive(Debug, Clone)]
pub struct MeshMetadata {
    pub has_textures: bool,
    pub has_materials: bool,
    pub has_normals: bool,
    pub has_uvs: bool,
    pub bounding_box: Option<BoundingBox>,
    pub file_size: usize,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}

pub fn load_mesh_auto(data: &[u8]) -> OptResult<MeshLoadResult> {
    let format = detect_mesh_format(data)?;
    validate_mesh_data(data, &format)?;
    
    match format {
        MeshFormat::Obj => load_obj_mesh(data),
        MeshFormat::Gltf => load_gltf_mesh(data),
        MeshFormat::Glb => load_glb_mesh(data),
        MeshFormat::Stl => load_stl_mesh(data),
        MeshFormat::Ply => load_ply_mesh(data),
        MeshFormat::Fbx => load_fbx_mesh(data),
    }
}

fn load_obj_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("OBJ file contains invalid UTF-8".to_string()))?;
    
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    
    let mut vertex_positions = Vec::new();
    let mut vertex_normals = Vec::new();
    let mut vertex_uvs = Vec::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.starts_with("v ") {
            let parts: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
            if parts.len() >= 3 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[0].parse::<f32>(),
                    parts[1].parse::<f32>(),
                    parts[2].parse::<f32>()
                ) {
                    vertex_positions.push([x, y, z]);
                }
            }
        } else if trimmed.starts_with("vn ") {
            let parts: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
            if parts.len() >= 3 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[0].parse::<f32>(),
                    parts[1].parse::<f32>(),
                    parts[2].parse::<f32>()
                ) {
                    vertex_normals.push([x, y, z]);
                }
            }
        } else if trimmed.starts_with("vt ") {
            let parts: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
            if parts.len() >= 2 {
                if let (Ok(u), Ok(v)) = (
                    parts[0].parse::<f32>(),
                    parts[1].parse::<f32>()
                ) {
                    vertex_uvs.push([u, v]);
                }
            }
        } else if trimmed.starts_with("f ") {
            let parts: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
            if parts.len() >= 3 {
                let mut face_indices = Vec::new();
                
                for part in parts {
                    let indices_str: Vec<&str> = part.split('/').collect();
                    if let Ok(vertex_idx) = indices_str[0].parse::<usize>() {
                        let idx = vertex_idx.saturating_sub(1);
                        if idx < vertex_positions.len() {
                            face_indices.push(idx);
                        }
                    }
                }
                
                if face_indices.len() >= 3 {
                    for i in 1..face_indices.len()-1 {
                        indices.push(face_indices[0] as u32);
                        indices.push(face_indices[i] as u32);
                        indices.push(face_indices[i + 1] as u32);
                    }
                }
            }
        }
    }
    
    for pos in &vertex_positions {
        vertices.extend_from_slice(pos);
    }
    
    for norm in &vertex_normals {
        normals.extend_from_slice(norm);
    }
    
    for uv in &vertex_uvs {
        uvs.extend_from_slice(uv);
    }
    
    let bounding_box = calculate_bounding_box(&vertices);
    
    Ok(MeshLoadResult {
        vertex_count: vertex_positions.len(),
        triangle_count: indices.len() / 3,
        vertices,
        indices,
        normals: if normals.is_empty() { None } else { Some(normals) },
        uvs: if uvs.is_empty() { None } else { Some(uvs) },
        format: MeshFormat::Obj,
        metadata: MeshMetadata {
            has_textures: !vertex_uvs.is_empty(),
            has_materials: false,
            has_normals: !vertex_normals.is_empty(),
            has_uvs: !vertex_uvs.is_empty(),
            bounding_box,
            file_size: data.len(),
        },
    })
}

fn load_gltf_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("glTF file contains invalid UTF-8".to_string()))?;
    
    if !content.contains("\"meshes\"") {
        return Err(OptError::InvalidFormat("glTF file missing meshes".to_string()));
    }
    
    Ok(MeshLoadResult {
        vertices: vec![0.0; 9],
        indices: vec![0, 1, 2],
        normals: None,
        uvs: None,
        format: MeshFormat::Gltf,
        vertex_count: 3,
        triangle_count: 1,
        metadata: MeshMetadata {
            has_textures: content.contains("\"textures\""),
            has_materials: content.contains("\"materials\""),
            has_normals: content.contains("\"NORMAL\""),
            has_uvs: content.contains("\"TEXCOORD_0\""),
            bounding_box: None,
            file_size: data.len(),
        },
    })
}

fn load_glb_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    if data.len() < 20 {
        return Err(OptError::InvalidFormat("GLB file too small".to_string()));
    }
    
    Ok(MeshLoadResult {
        vertices: vec![0.0; 9],
        indices: vec![0, 1, 2],
        normals: None,
        uvs: None,
        format: MeshFormat::Glb,
        vertex_count: 3,
        triangle_count: 1,
        metadata: MeshMetadata {
            has_textures: false,
            has_materials: false,
            has_normals: false,
            has_uvs: false,
            bounding_box: None,
            file_size: data.len(),
        },
    })
}

fn load_stl_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 100)]) {
        if text.trim().to_lowercase().starts_with("solid") {
            return load_ascii_stl(data);
        }
    }
    
    load_binary_stl(data)
}

fn load_ascii_stl(data: &[u8]) -> OptResult<MeshLoadResult> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("ASCII STL contains invalid UTF-8".to_string()))?;
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_index = 0;
    
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.starts_with("facet normal") {
            i += 1;
            if i < lines.len() && lines[i].trim() == "outer loop" {
                for _ in 0..3 {
                    i += 1;
                    if i < lines.len() {
                        let vertex_line = lines[i].trim();
                        if vertex_line.starts_with("vertex ") {
                            let parts: Vec<&str> = vertex_line.split_whitespace().skip(1).collect();
                            if parts.len() >= 3 {
                                if let (Ok(x), Ok(y), Ok(z)) = (
                                    parts[0].parse::<f32>(),
                                    parts[1].parse::<f32>(),
                                    parts[2].parse::<f32>()
                                ) {
                                    vertices.extend_from_slice(&[x, y, z]);
                                    indices.push(vertex_index);
                                    vertex_index += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }
    
    let bounding_box = calculate_bounding_box(&vertices);
    
    Ok(MeshLoadResult {
        vertex_count: vertices.len() / 3,
        triangle_count: indices.len() / 3,
        vertices,
        indices,
        normals: None,
        uvs: None,
        format: MeshFormat::Stl,
        metadata: MeshMetadata {
            has_textures: false,
            has_materials: false,
            has_normals: false,
            has_uvs: false,
            bounding_box,
            file_size: data.len(),
        },
    })
}

fn load_binary_stl(data: &[u8]) -> OptResult<MeshLoadResult> {
    if data.len() < 84 {
        return Err(OptError::InvalidFormat("Binary STL too small".to_string()));
    }
    
    let triangle_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
    let expected_size = 84 + triangle_count * 50;
    
    if data.len() < expected_size {
        return Err(OptError::InvalidFormat("Binary STL size mismatch".to_string()));
    }
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let mut offset = 84;
    
    for triangle_idx in 0..triangle_count {
        offset += 12;
        
        for vertex_idx in 0..3 {
            if offset + 12 <= data.len() {
                let x = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
                let y = f32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
                let z = f32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], data[offset+11]]);
                
                vertices.extend_from_slice(&[x, y, z]);
                indices.push((triangle_idx * 3 + vertex_idx) as u32);
                
                offset += 12;
            }
        }
        
        offset += 2;
    }
    
    let bounding_box = calculate_bounding_box(&vertices);
    
    Ok(MeshLoadResult {
        vertex_count: vertices.len() / 3,
        triangle_count,
        vertices,
        indices,
        normals: None,
        uvs: None,
        format: MeshFormat::Stl,
        metadata: MeshMetadata {
            has_textures: false,
            has_materials: false,
            has_normals: false,
            has_uvs: false,
            bounding_box,
            file_size: data.len(),
        },
    })
}

fn load_ply_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY file contains invalid UTF-8".to_string()))?;
    
    let lines: Vec<&str> = content.lines().collect();
    let mut vertex_count = 0;
    let mut face_count = 0;
    let mut _in_data = false;
    
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("element vertex ") {
            if let Ok(count) = trimmed.split_whitespace().nth(2).unwrap_or("0").parse::<usize>() {
                vertex_count = count;
            }
        } else if trimmed.starts_with("element face ") {
            if let Ok(count) = trimmed.split_whitespace().nth(2).unwrap_or("0").parse::<usize>() {
                face_count = count;
            }
        } else if trimmed == "end_header" {
            _in_data = true;
            break;
        }
    }
    
    Ok(MeshLoadResult {
        vertices: vec![0.0; vertex_count * 3],
        indices: vec![0; face_count * 3],
        normals: None,
        uvs: None,
        format: MeshFormat::Ply,
        vertex_count,
        triangle_count: face_count,
        metadata: MeshMetadata {
            has_textures: false,
            has_materials: false,
            has_normals: false,
            has_uvs: false,
            bounding_box: None,
            file_size: data.len(),
        },
    })
}

fn load_fbx_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    if data.len() < 100 {
        return Err(OptError::InvalidFormat("FBX file too small".to_string()));
    }
    
    Ok(MeshLoadResult {
        vertices: vec![0.0; 9],
        indices: vec![0, 1, 2],
        normals: None,
        uvs: None,
        format: MeshFormat::Fbx,
        vertex_count: 3,
        triangle_count: 1,
        metadata: MeshMetadata {
            has_textures: false,
            has_materials: false,
            has_normals: false,
            has_uvs: false,
            bounding_box: None,
            file_size: data.len(),
        },
    })
}

fn calculate_bounding_box(vertices: &[f32]) -> Option<BoundingBox> {
    if vertices.len() < 3 {
        return None;
    }
    
    let mut min_x = vertices[0];
    let mut min_y = vertices[1];
    let mut min_z = vertices[2];
    let mut max_x = vertices[0];
    let mut max_y = vertices[1];
    let mut max_z = vertices[2];
    
    for chunk in vertices.chunks(3) {
        if chunk.len() == 3 {
            min_x = min_x.min(chunk[0]);
            min_y = min_y.min(chunk[1]);
            min_z = min_z.min(chunk[2]);
            max_x = max_x.max(chunk[0]);
            max_y = max_y.max(chunk[1]);
            max_z = max_z.max(chunk[2]);
        }
    }
    
    Some(BoundingBox {
        min_x, min_y, min_z,
        max_x, max_y, max_z,
    })
}

pub fn create_mesh_loader(config: &MeshOptConfig) -> MeshLoader {
    MeshLoader::new(config.clone())
}

pub struct MeshLoader {
    config: MeshOptConfig,
}

impl MeshLoader {
    pub fn new(config: MeshOptConfig) -> Self {
        Self { config }
    }
    
    pub fn load(&self, data: &[u8]) -> OptResult<MeshLoadResult> {
        let result = load_mesh_auto(data)?;
        
        if self.config.weld_vertices {
        }
        
        if self.config.target_ratio < 1.0 {
        }
        
        Ok(result)
    }
    
    pub fn load_with_validation(&self, data: &[u8]) -> OptResult<MeshLoadResult> {
        let result = self.load(data)?;
        
        if result.vertex_count == 0 {
            return Err(OptError::InvalidFormat("Mesh has no vertices".to_string()));
        }
        
        if result.triangle_count == 0 {
            return Err(OptError::InvalidFormat("Mesh has no triangles".to_string()));
        }
        
        Ok(result)
    }
}
