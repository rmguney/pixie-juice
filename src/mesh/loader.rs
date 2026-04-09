extern crate alloc;
use alloc::{vec::Vec, string::ToString};
use crate::types::{OptResult, OptError, MeshOptConfig};
use crate::formats::MeshFormat;
use crate::mesh::formats::{detect_mesh_format, validate_mesh_data};
use crate::mesh::ply::load_ply_mesh;
use crate::mesh::fbx::load_fbx_mesh;

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

    let mut vertex_positions: Vec<[f32; 3]> = Vec::new();
    let mut vertex_normals: Vec<[f32; 3]> = Vec::new();
    let mut vertex_uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut has_materials = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("v ") {
            let parts: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
            if parts.len() >= 3 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[0].parse::<f32>(),
                    parts[1].parse::<f32>(),
                    parts[2].parse::<f32>(),
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
                    parts[2].parse::<f32>(),
                ) {
                    vertex_normals.push([x, y, z]);
                }
            }
        } else if trimmed.starts_with("vt ") {
            let parts: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
            if parts.len() >= 2 {
                if let (Ok(u), Ok(v)) = (
                    parts[0].parse::<f32>(),
                    parts[1].parse::<f32>(),
                ) {
                    vertex_uvs.push([u, v]);
                }
            }
        } else if trimmed.starts_with("f ") {
            let parts: Vec<&str> = trimmed.split_whitespace().skip(1).collect();
            if parts.len() >= 3 {
                let mut face_indices: Vec<usize> = Vec::new();
                for part in parts {
                    let indices_str: Vec<&str> = part.split('/').collect();
                    if let Ok(vertex_idx) = indices_str[0].parse::<i32>() {
                        let idx = if vertex_idx > 0 {
                            (vertex_idx - 1) as usize
                        } else if vertex_idx < 0 {
                            vertex_positions.len().saturating_sub((-vertex_idx) as usize)
                        } else {
                            continue;
                        };
                        if idx < vertex_positions.len() {
                            face_indices.push(idx);
                        }
                    }
                }
                if face_indices.len() >= 3 {
                    for i in 1..face_indices.len() - 1 {
                        indices.push(face_indices[0] as u32);
                        indices.push(face_indices[i] as u32);
                        indices.push(face_indices[i + 1] as u32);
                    }
                }
            }
        } else if trimmed.starts_with("usemtl ") || trimmed.starts_with("mtllib ") {
            has_materials = true;
        }
    }

    let mut vertices = Vec::with_capacity(vertex_positions.len() * 3);
    for pos in &vertex_positions {
        vertices.extend_from_slice(pos);
    }
    let mut normals = Vec::with_capacity(vertex_normals.len() * 3);
    for n in &vertex_normals {
        normals.extend_from_slice(n);
    }
    let mut uvs = Vec::with_capacity(vertex_uvs.len() * 2);
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
            has_materials,
            has_normals: !vertex_normals.is_empty(),
            has_uvs: !vertex_uvs.is_empty(),
            bounding_box,
            file_size: data.len(),
        },
    })
}

#[cfg(feature = "gltf")]
fn load_gltf_from_bytes(data: &[u8], format: MeshFormat) -> OptResult<MeshLoadResult> {
    let gltf = gltf::Gltf::from_slice(data)
        .map_err(|e| OptError::InvalidFormat(alloc::format!("glTF parse error: {}", e)))?;

    let blob = gltf.blob.as_deref();
    let mut buffer_data: Vec<Vec<u8>> = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                if let Some(b) = blob {
                    buffer_data.push(b.to_vec());
                } else {
                    return Err(OptError::InvalidFormat("glTF expects binary buffer but none present".to_string()));
                }
            }
            gltf::buffer::Source::Uri(uri) => {
                if let Some(stripped) = uri.strip_prefix("data:") {
                    if let Some(idx) = stripped.find(";base64,") {
                        let b64 = &stripped[idx + 8..];
                        buffer_data.push(decode_base64(b64)?);
                    } else {
                        return Err(OptError::InvalidFormat("glTF non-base64 data URI not supported".to_string()));
                    }
                } else {
                    return Err(OptError::InvalidFormat("glTF external buffer URIs not supported in browser-only mode".to_string()));
                }
            }
        }
    }

    let mut positions: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut index_offset: u32 = 0;
    let mut has_textures = false;
    let mut has_materials = false;

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            if primitive.material().index().is_some() {
                has_materials = true;
            }
            let reader = primitive.reader(|buffer| buffer_data.get(buffer.index()).map(|v| v.as_slice()));

            let mut primitive_vertex_count: u32 = 0;
            if let Some(iter) = reader.read_positions() {
                for pos in iter {
                    positions.push(pos[0]);
                    positions.push(pos[1]);
                    positions.push(pos[2]);
                    primitive_vertex_count += 1;
                }
            }

            if let Some(iter) = reader.read_normals() {
                for n in iter {
                    normals.push(n[0]);
                    normals.push(n[1]);
                    normals.push(n[2]);
                }
            }

            if let Some(read_tex) = reader.read_tex_coords(0) {
                has_textures = true;
                for uv in read_tex.into_f32() {
                    uvs.push(uv[0]);
                    uvs.push(uv[1]);
                }
            }

            if let Some(read_idx) = reader.read_indices() {
                for idx in read_idx.into_u32() {
                    indices.push(idx + index_offset);
                }
            } else {
                for i in 0..primitive_vertex_count {
                    indices.push(index_offset + i);
                }
            }

            index_offset += primitive_vertex_count;
        }
    }

    if positions.is_empty() {
        return Err(OptError::InvalidFormat("glTF contains no vertex positions".to_string()));
    }

    let bounding_box = calculate_bounding_box(&positions);
    let vertex_count = positions.len() / 3;
    let triangle_count = indices.len() / 3;
    let has_normals = !normals.is_empty();
    let has_uvs = !uvs.is_empty();

    Ok(MeshLoadResult {
        vertices: positions,
        indices,
        normals: if has_normals { Some(normals) } else { None },
        uvs: if has_uvs { Some(uvs) } else { None },
        format,
        vertex_count,
        triangle_count,
        metadata: MeshMetadata {
            has_textures,
            has_materials,
            has_normals,
            has_uvs,
            bounding_box,
            file_size: data.len(),
        },
    })
}

#[cfg(feature = "gltf")]
fn decode_base64(input: &str) -> OptResult<Vec<u8>> {
    let cleaned: alloc::string::String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = cleaned.as_bytes();
    if bytes.is_empty() {
        return Ok(Vec::new());
    }

    if bytes.len() % 4 != 0 {
        return Err(OptError::InvalidFormat("base64 length not a multiple of 4".to_string()));
    }

    fn val(c: u8) -> Result<u8, OptError> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0),
            _ => Err(OptError::InvalidFormat("invalid base64 character".to_string())),
        }
    }

    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let b0 = val(chunk[0])?;
        let b1 = val(chunk[1])?;
        let b2 = val(chunk[2])?;
        let b3 = val(chunk[3])?;

        out.push((b0 << 2) | (b1 >> 4));
        if chunk[2] != b'=' {
            out.push((b1 << 4) | (b2 >> 2));
        }
        if chunk[3] != b'=' {
            out.push((b2 << 6) | b3);
        }
    }

    Ok(out)
}

#[cfg(not(feature = "gltf"))]
fn load_gltf_from_bytes(_data: &[u8], _format: MeshFormat) -> OptResult<MeshLoadResult> {
    Err(OptError::InvalidFormat("glTF support not compiled".to_string()))
}

fn load_gltf_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    load_gltf_from_bytes(data, MeshFormat::Gltf)
}

fn load_glb_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    load_gltf_from_bytes(data, MeshFormat::Glb)
}

fn load_stl_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    if let Ok(text) = core::str::from_utf8(&data[..core::cmp::min(data.len(), 100)]) {
        if text.trim_start().to_lowercase().starts_with("solid") && !is_binary_stl_disguised(data) {
            return load_ascii_stl(data);
        }
    }
    load_binary_stl(data)
}

fn is_binary_stl_disguised(data: &[u8]) -> bool {
    if data.len() < 84 {
        return false;
    }
    let triangle_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
    let expected_size = 84usize.saturating_add(triangle_count.saturating_mul(50));
    expected_size == data.len()
}

fn load_ascii_stl(data: &[u8]) -> OptResult<MeshLoadResult> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("ASCII STL contains invalid UTF-8".to_string()))?;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_index: u32 = 0;

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
                        if let Some(rest) = vertex_line.strip_prefix("vertex ") {
                            let parts: Vec<&str> = rest.split_whitespace().collect();
                            if parts.len() >= 3 {
                                if let (Ok(x), Ok(y), Ok(z)) = (
                                    parts[0].parse::<f32>(),
                                    parts[1].parse::<f32>(),
                                    parts[2].parse::<f32>(),
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
    let expected_size = 84usize.checked_add(triangle_count.checked_mul(50).unwrap_or(usize::MAX))
        .unwrap_or(usize::MAX);
    if data.len() < expected_size {
        return Err(OptError::InvalidFormat("Binary STL size mismatch".to_string()));
    }

    let mut vertices = Vec::with_capacity(triangle_count * 9);
    let mut indices = Vec::with_capacity(triangle_count * 3);

    let mut offset = 84;
    for triangle_idx in 0..triangle_count {
        offset += 12;
        for vertex_idx in 0..3 {
            if offset + 12 > data.len() {
                break;
            }
            let x = f32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
            let y = f32::from_le_bytes([data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]]);
            let z = f32::from_le_bytes([data[offset + 8], data[offset + 9], data[offset + 10], data[offset + 11]]);
            vertices.extend_from_slice(&[x, y, z]);
            indices.push((triangle_idx * 3 + vertex_idx) as u32);
            offset += 12;
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

pub(crate) fn calculate_bounding_box(vertices: &[f32]) -> Option<BoundingBox> {
    if vertices.len() < 3 {
        return None;
    }
    let mut min_x = vertices[0];
    let mut min_y = vertices[1];
    let mut min_z = vertices[2];
    let mut max_x = vertices[0];
    let mut max_y = vertices[1];
    let mut max_z = vertices[2];

    for chunk in vertices.chunks_exact(3) {
        if chunk[0] < min_x { min_x = chunk[0]; }
        if chunk[1] < min_y { min_y = chunk[1]; }
        if chunk[2] < min_z { min_z = chunk[2]; }
        if chunk[0] > max_x { max_x = chunk[0]; }
        if chunk[1] > max_y { max_y = chunk[1]; }
        if chunk[2] > max_z { max_z = chunk[2]; }
    }

    Some(BoundingBox { min_x, min_y, min_z, max_x, max_y, max_z })
}

pub fn create_mesh_loader(config: &MeshOptConfig) -> MeshLoader {
    MeshLoader::new(config.clone())
}

pub struct MeshLoader {
    _config: MeshOptConfig,
}

impl MeshLoader {
    pub fn new(config: MeshOptConfig) -> Self {
        Self { _config: config }
    }

    pub fn load(&self, data: &[u8]) -> OptResult<MeshLoadResult> {
        load_mesh_auto(data)
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
