extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}, format};

use crate::types::{MeshOptConfig, OptResult, OptError};
use crate::mesh::loader::{MeshLoadResult, MeshMetadata, calculate_bounding_box};
use crate::formats::MeshFormat;

pub fn optimize_ply(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if !data.starts_with(b"ply") {
        return Err(OptError::InvalidFormat("Not a valid PLY file".to_string()));
    }

    if is_binary_ply(data) {
        return optimize_binary_ply(data, config);
    }

    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY file contains invalid UTF-8".to_string()))?;

    let optimized_content = if config.weld_vertices {
        optimize_ply_geometry(content, config)?
    } else {
        remove_ply_comments(content)?
    };

    Ok(optimized_content.into_bytes())
}

fn is_binary_ply(data: &[u8]) -> bool {
    let header_end_marker = b"end_header";
    let header_limit = data.len().min(8192);
    let search_window = &data[..header_limit];
    let header_end = search_window
        .windows(header_end_marker.len())
        .position(|w| w == header_end_marker);
    let header_bytes = match header_end {
        Some(end) => &search_window[..end],
        None => search_window,
    };

    let header_text = match core::str::from_utf8(header_bytes) {
        Ok(text) => text,
        Err(_) => return false,
    };

    header_text
        .lines()
        .any(|line| line.trim().starts_with("format binary"))
}

fn optimize_binary_ply(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    let mut header_end = 0;

    #[cfg(c_hotspots_available)]
    if config.use_c_hotspots {
        if let Some(pos) = crate::c_hotspots::util::ply_find_end_header(data) {
            header_end = pos;
        }
    }

    if header_end == 0 {
        if let Ok(text) = core::str::from_utf8(data) {
            if let Some(pos) = text.find("end_header") {
                let end_header_line_end = text[pos..].find('\n').unwrap_or(0);
                header_end = pos + end_header_line_end + 1;
            }
        } else {
            let end_header_bytes = b"end_header";
            for i in 0..data.len().saturating_sub(end_header_bytes.len()) {
                if &data[i..i + end_header_bytes.len()] == end_header_bytes {
                    for j in i + end_header_bytes.len()..data.len() {
                        if data[j] == b'\n' {
                            header_end = j + 1;
                            break;
                        }
                    }
                    break;
                }
            }
        }
    }

    if header_end == 0 {
        return Err(OptError::InvalidFormat("Binary PLY: could not find end_header".to_string()));
    }

    let mut optimized_data = data[0..header_end].to_vec();
    let binary_data = &data[header_end..];

    let compression_ratio = config.target_ratio.max(0.5);
    let compressed_size = (binary_data.len() as f32 * compression_ratio) as usize;
    let compressed_binary = if compressed_size < binary_data.len() {
        &binary_data[0..compressed_size]
    } else {
        binary_data
    };

    optimized_data.extend_from_slice(compressed_binary);
    Ok(optimized_data)
}

fn remove_ply_comments(content: &str) -> OptResult<String> {
    let lines: Vec<&str> = content.lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && !trimmed.is_empty()
        })
        .collect();
    Ok(lines.join("\n"))
}

fn optimize_ply_geometry(content: &str, config: &MeshOptConfig) -> OptResult<String> {
    let mut result_lines = Vec::new();
    let mut in_vertex_data = false;
    let mut vertex_count = 0usize;
    let mut processed_vertices = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("element vertex ") {
            if let Some(count_str) = trimmed.split_whitespace().nth(2) {
                vertex_count = count_str.parse().unwrap_or(0);
            }
            result_lines.push(line.to_string());
            continue;
        }

        if trimmed == "end_header" {
            in_vertex_data = true;
            result_lines.push(line.to_string());
            continue;
        }

        if in_vertex_data && processed_vertices < vertex_count {
            let optimized_vertex = optimize_vertex_line(line, config)?;
            result_lines.push(optimized_vertex);
            processed_vertices += 1;

            if processed_vertices >= vertex_count {
                in_vertex_data = false;
            }
            continue;
        }

        result_lines.push(line.to_string());
    }

    Ok(result_lines.join("\n"))
}

fn optimize_vertex_line(line: &str, config: &MeshOptConfig) -> OptResult<String> {
    let values: Vec<&str> = line.split_whitespace().collect();
    let mut optimized_values = Vec::new();

    for value in values {
        if let Ok(f) = value.parse::<f32>() {
            let factor = 1.0 / config.vertex_tolerance;
            let rounded = (f * factor).round() / factor;
            optimized_values.push(format!("{:.6}", rounded).trim_end_matches('0').trim_end_matches('.').to_string());
        } else {
            optimized_values.push(value.to_string());
        }
    }

    Ok(optimized_values.join(" "))
}

pub fn validate_ply_structure(data: &[u8]) -> OptResult<bool> {
    if is_binary_ply(data) {
        if let Ok(header_text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 500)]) {
            return Ok(header_text.starts_with("ply") && header_text.contains("end_header"));
        } else {
            return Ok(data.starts_with(b"ply") &&
                     data.windows(10).any(|window| window == b"end_header"));
        }
    }

    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY file contains invalid UTF-8".to_string()))?;

    if !content.starts_with("ply") {
        return Err(OptError::InvalidFormat("Not a valid PLY file".to_string()));
    }

    if !content.contains("format") {
        return Err(OptError::InvalidFormat("PLY missing format declaration".to_string()));
    }

    if !content.contains("end_header") {
        return Err(OptError::InvalidFormat("PLY missing end_header".to_string()));
    }

    Ok(true)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlyType {
    Char, Uchar, Short, Ushort, Int, Uint, Float, Double,
}

impl PlyType {
    fn from_name(s: &str) -> Option<PlyType> {
        Some(match s {
            "char" | "int8" => PlyType::Char,
            "uchar" | "uint8" => PlyType::Uchar,
            "short" | "int16" => PlyType::Short,
            "ushort" | "uint16" => PlyType::Ushort,
            "int" | "int32" => PlyType::Int,
            "uint" | "uint32" => PlyType::Uint,
            "float" | "float32" => PlyType::Float,
            "double" | "float64" => PlyType::Double,
            _ => return None,
        })
    }
    fn size(self) -> usize {
        match self {
            PlyType::Char | PlyType::Uchar => 1,
            PlyType::Short | PlyType::Ushort => 2,
            PlyType::Int | PlyType::Uint | PlyType::Float => 4,
            PlyType::Double => 8,
        }
    }
}

#[derive(Debug, Clone)]
struct PlyProperty {
    name: String,
    scalar_type: PlyType,
    list_count_type: Option<PlyType>,
}

#[derive(Debug, Clone)]
struct PlyElement {
    name: String,
    count: usize,
    properties: Vec<PlyProperty>,
}

#[derive(Debug, Clone)]
struct PlyHeader {
    is_binary: bool,
    is_little_endian: bool,
    elements: Vec<PlyElement>,
    data_offset: usize,
}

pub fn load_ply_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    let header = parse_ply_header(data)?;
    if header.is_binary {
        load_ply_binary(data, &header)
    } else {
        load_ply_ascii(data, &header)
    }
}

fn parse_ply_header(data: &[u8]) -> OptResult<PlyHeader> {
    let header_end_marker = b"end_header";
    let mut header_end = None;
    for i in 0..data.len().saturating_sub(header_end_marker.len()) {
        if &data[i..i + header_end_marker.len()] == header_end_marker {
            let mut after = i + header_end_marker.len();
            while after < data.len() && (data[after] == b'\r' || data[after] == b'\n') {
                after += 1;
            }
            header_end = Some(after);
            break;
        }
    }
    let data_offset = header_end.ok_or_else(|| OptError::InvalidFormat("PLY missing end_header".to_string()))?;

    let header_bytes = &data[..data_offset];
    let header_text = core::str::from_utf8(header_bytes)
        .map_err(|_| OptError::InvalidFormat("PLY header is not valid UTF-8".to_string()))?;

    if !header_text.starts_with("ply") {
        return Err(OptError::InvalidFormat("PLY missing magic".to_string()));
    }

    let mut is_binary = false;
    let mut is_little_endian = true;
    let mut elements: Vec<PlyElement> = Vec::new();

    for line in header_text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("format ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            match parts.first().copied() {
                Some("ascii") => is_binary = false,
                Some("binary_little_endian") => { is_binary = true; is_little_endian = true; }
                Some("binary_big_endian") => { is_binary = true; is_little_endian = false; }
                Some(other) => return Err(OptError::InvalidFormat(format!("Unknown PLY format: {}", other))),
                None => {}
            }
        } else if let Some(rest) = trimmed.strip_prefix("element ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                let count = parts[1].parse::<usize>()
                    .map_err(|_| OptError::InvalidFormat("PLY element count parse error".to_string()))?;
                elements.push(PlyElement {
                    name: parts[0].to_string(),
                    count,
                    properties: Vec::new(),
                });
            }
        } else if let Some(rest) = trimmed.strip_prefix("property ") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            let current = elements.last_mut()
                .ok_or_else(|| OptError::InvalidFormat("PLY property before element".to_string()))?;
            if parts[0] == "list" {
                if parts.len() < 4 {
                    return Err(OptError::InvalidFormat("PLY list property malformed".to_string()));
                }
                let count_type = PlyType::from_name(parts[1])
                    .ok_or_else(|| OptError::InvalidFormat(format!("Unknown PLY type: {}", parts[1])))?;
                let scalar_type = PlyType::from_name(parts[2])
                    .ok_or_else(|| OptError::InvalidFormat(format!("Unknown PLY type: {}", parts[2])))?;
                current.properties.push(PlyProperty {
                    name: parts[3].to_string(),
                    scalar_type,
                    list_count_type: Some(count_type),
                });
            } else {
                if parts.len() < 2 {
                    return Err(OptError::InvalidFormat("PLY property malformed".to_string()));
                }
                let scalar_type = PlyType::from_name(parts[0])
                    .ok_or_else(|| OptError::InvalidFormat(format!("Unknown PLY type: {}", parts[0])))?;
                current.properties.push(PlyProperty {
                    name: parts[1].to_string(),
                    scalar_type,
                    list_count_type: None,
                });
            }
        }
    }

    Ok(PlyHeader { is_binary, is_little_endian, elements, data_offset })
}

fn read_ply_scalar(data: &[u8], offset: &mut usize, ty: PlyType, little_endian: bool) -> OptResult<f64> {
    let size = ty.size();
    if *offset + size > data.len() {
        return Err(OptError::InvalidFormat("PLY binary data truncated".to_string()));
    }
    let bytes = &data[*offset..*offset + size];
    let value = match ty {
        PlyType::Char => bytes[0] as i8 as f64,
        PlyType::Uchar => bytes[0] as f64,
        PlyType::Short => if little_endian { i16::from_le_bytes([bytes[0], bytes[1]]) as f64 }
                         else { i16::from_be_bytes([bytes[0], bytes[1]]) as f64 },
        PlyType::Ushort => if little_endian { u16::from_le_bytes([bytes[0], bytes[1]]) as f64 }
                          else { u16::from_be_bytes([bytes[0], bytes[1]]) as f64 },
        PlyType::Int => if little_endian { i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64 }
                       else { i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64 },
        PlyType::Uint => if little_endian { u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64 }
                        else { u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64 },
        PlyType::Float => if little_endian { f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64 }
                         else { f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64 },
        PlyType::Double => if little_endian { f64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]) }
                          else { f64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]) },
    };
    *offset += size;
    Ok(value)
}

fn load_ply_binary(data: &[u8], header: &PlyHeader) -> OptResult<MeshLoadResult> {
    let mut offset = header.data_offset;
    let mut vertices: Vec<f32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for element in &header.elements {
        if element.name == "vertex" {
            let pos_idx = find_xyz_indices(&element.properties);
            let nrm_idx = find_nxyz_indices(&element.properties);
            let uv_idx = find_uv_indices(&element.properties);

            for _ in 0..element.count {
                let mut row = Vec::with_capacity(element.properties.len());
                for prop in &element.properties {
                    if let Some(count_ty) = prop.list_count_type {
                        let n = read_ply_scalar(data, &mut offset, count_ty, header.is_little_endian)? as usize;
                        let mut last = 0.0;
                        for _ in 0..n {
                            last = read_ply_scalar(data, &mut offset, prop.scalar_type, header.is_little_endian)?;
                        }
                        row.push(last);
                    } else {
                        row.push(read_ply_scalar(data, &mut offset, prop.scalar_type, header.is_little_endian)?);
                    }
                }
                if let Some((x, y, z)) = pos_idx {
                    vertices.push(row[x] as f32);
                    vertices.push(row[y] as f32);
                    vertices.push(row[z] as f32);
                }
                if let Some((x, y, z)) = nrm_idx {
                    normals.push(row[x] as f32);
                    normals.push(row[y] as f32);
                    normals.push(row[z] as f32);
                }
                if let Some((u, v)) = uv_idx {
                    uvs.push(row[u] as f32);
                    uvs.push(row[v] as f32);
                }
            }
        } else if element.name == "face" || element.name == "tristrips" {
            for _ in 0..element.count {
                for prop in &element.properties {
                    if let Some(count_ty) = prop.list_count_type {
                        let n = read_ply_scalar(data, &mut offset, count_ty, header.is_little_endian)? as usize;
                        let mut face: Vec<u32> = Vec::with_capacity(n);
                        for _ in 0..n {
                            let v = read_ply_scalar(data, &mut offset, prop.scalar_type, header.is_little_endian)? as u32;
                            face.push(v);
                        }
                        if face.len() >= 3 && prop.name == "vertex_indices" {
                            for i in 1..face.len() - 1 {
                                indices.push(face[0]);
                                indices.push(face[i]);
                                indices.push(face[i + 1]);
                            }
                        }
                    } else {
                        let _ = read_ply_scalar(data, &mut offset, prop.scalar_type, header.is_little_endian)?;
                    }
                }
            }
        } else {
            for _ in 0..element.count {
                for prop in &element.properties {
                    if let Some(count_ty) = prop.list_count_type {
                        let n = read_ply_scalar(data, &mut offset, count_ty, header.is_little_endian)? as usize;
                        for _ in 0..n {
                            let _ = read_ply_scalar(data, &mut offset, prop.scalar_type, header.is_little_endian)?;
                        }
                    } else {
                        let _ = read_ply_scalar(data, &mut offset, prop.scalar_type, header.is_little_endian)?;
                    }
                }
            }
        }
    }

    finalize_ply(data, vertices, indices, normals, uvs)
}

fn load_ply_ascii(data: &[u8], header: &PlyHeader) -> OptResult<MeshLoadResult> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY ASCII contains invalid UTF-8".to_string()))?;
    let body = &content[header.data_offset.min(content.len())..];

    let mut vertices: Vec<f32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut lines = body.lines();
    for element in &header.elements {
        let pos_idx = find_xyz_indices(&element.properties);
        let nrm_idx = find_nxyz_indices(&element.properties);
        let uv_idx = find_uv_indices(&element.properties);

        for _ in 0..element.count {
            let line = match lines.next() {
                Some(l) => l,
                None => return Err(OptError::InvalidFormat("PLY ASCII data truncated".to_string())),
            };
            let mut tokens = line.split_whitespace();

            if element.name == "vertex" {
                let mut row: Vec<f64> = Vec::with_capacity(element.properties.len());
                for prop in &element.properties {
                    if prop.list_count_type.is_some() {
                        let n = tokens.next().and_then(|t| t.parse::<usize>().ok()).unwrap_or(0);
                        let mut last = 0.0f64;
                        for _ in 0..n {
                            last = tokens.next().and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.0);
                        }
                        row.push(last);
                    } else {
                        let v = tokens.next().and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.0);
                        row.push(v);
                    }
                }
                if let Some((x, y, z)) = pos_idx {
                    vertices.push(row[x] as f32);
                    vertices.push(row[y] as f32);
                    vertices.push(row[z] as f32);
                }
                if let Some((x, y, z)) = nrm_idx {
                    normals.push(row[x] as f32);
                    normals.push(row[y] as f32);
                    normals.push(row[z] as f32);
                }
                if let Some((u, v)) = uv_idx {
                    uvs.push(row[u] as f32);
                    uvs.push(row[v] as f32);
                }
            } else if element.name == "face" {
                for prop in &element.properties {
                    if prop.list_count_type.is_some() && prop.name == "vertex_indices" {
                        let n = tokens.next().and_then(|t| t.parse::<usize>().ok()).unwrap_or(0);
                        let mut face: Vec<u32> = Vec::with_capacity(n);
                        for _ in 0..n {
                            let v = tokens.next().and_then(|t| t.parse::<u32>().ok()).unwrap_or(0);
                            face.push(v);
                        }
                        if face.len() >= 3 {
                            for i in 1..face.len() - 1 {
                                indices.push(face[0]);
                                indices.push(face[i]);
                                indices.push(face[i + 1]);
                            }
                        }
                    } else if prop.list_count_type.is_some() {
                        let n = tokens.next().and_then(|t| t.parse::<usize>().ok()).unwrap_or(0);
                        for _ in 0..n { let _ = tokens.next(); }
                    } else {
                        let _ = tokens.next();
                    }
                }
            }
        }
    }

    finalize_ply(data, vertices, indices, normals, uvs)
}

fn find_xyz_indices(props: &[PlyProperty]) -> Option<(usize, usize, usize)> {
    let x = props.iter().position(|p| p.name == "x")?;
    let y = props.iter().position(|p| p.name == "y")?;
    let z = props.iter().position(|p| p.name == "z")?;
    Some((x, y, z))
}

fn find_nxyz_indices(props: &[PlyProperty]) -> Option<(usize, usize, usize)> {
    let x = props.iter().position(|p| p.name == "nx")?;
    let y = props.iter().position(|p| p.name == "ny")?;
    let z = props.iter().position(|p| p.name == "nz")?;
    Some((x, y, z))
}

fn find_uv_indices(props: &[PlyProperty]) -> Option<(usize, usize)> {
    let candidates = [("s", "t"), ("u", "v"), ("texture_u", "texture_v")];
    for (u_name, v_name) in candidates {
        let u = props.iter().position(|p| p.name == u_name);
        let v = props.iter().position(|p| p.name == v_name);
        if let (Some(u), Some(v)) = (u, v) {
            return Some((u, v));
        }
    }
    None
}

fn finalize_ply(
    data: &[u8],
    vertices: Vec<f32>,
    indices: Vec<u32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
) -> OptResult<MeshLoadResult> {
    let bounding_box = calculate_bounding_box(&vertices);
    let vertex_count = vertices.len() / 3;
    let triangle_count = indices.len() / 3;
    let has_normals = !normals.is_empty();
    let has_uvs = !uvs.is_empty();
    Ok(MeshLoadResult {
        vertices,
        indices,
        normals: if has_normals { Some(normals) } else { None },
        uvs: if has_uvs { Some(uvs) } else { None },
        format: MeshFormat::Ply,
        vertex_count,
        triangle_count,
        metadata: MeshMetadata {
            has_textures: has_uvs,
            has_materials: false,
            has_normals,
            has_uvs,
            bounding_box,
            file_size: data.len(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ply_optimization() {
        let config = MeshOptConfig::default();

        let ply_content = b"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
element face 1
property list uchar int vertex_indices
end_header
0.000000 0.000000 0.000000
1.000000 0.000000 0.000000
0.000000 1.000000 0.000000
3 0 1 2
";

        let result = optimize_ply(ply_content, &config);
        assert!(result.is_ok());

        let optimized = String::from_utf8(result.unwrap()).unwrap();
        assert!(!optimized.contains("0.000000"));
    }

    #[test]
    fn test_ply_validation() {
        let valid_ply = b"ply
format ascii 1.0
element vertex 1
property float x
property float y
property float z
end_header
0.0 0.0 0.0
";

        let result = validate_ply_structure(valid_ply);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let invalid_ply = b"not_a_ply_file";
        let result = validate_ply_structure(invalid_ply);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_ply_ascii() {
        let ply = b"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
element face 1
property list uchar int vertex_indices
end_header
0 0 0
1 0 0
0 1 0
3 0 1 2
";
        let result = load_ply_mesh(ply).expect("ASCII PLY parse");
        assert_eq!(result.vertex_count, 3);
        assert_eq!(result.triangle_count, 1);
    }

    #[test]
    fn test_binary_ply_detection_with_binary_payload_in_first_500_bytes() {
        // Header is short so byte 500 is well into the binary section. This used to
        // false-negative because is_binary_ply ran from_utf8 on the first 500 bytes
        // and bailed when the binary payload tripped UTF-8 validation.
        let header = b"ply\nformat binary_little_endian 1.0\nelement vertex 1\n\
                       property float x\nproperty float y\nproperty float z\n\
                       end_header\n";
        let mut data = header.to_vec();
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());
        data.extend_from_slice(&[0xFFu8; 480]);

        assert!(is_binary_ply(&data));
    }

    #[test]
    fn test_optimize_binary_ply_does_not_return_utf8_error() {
        let header = b"ply\nformat binary_little_endian 1.0\nelement vertex 2\n\
                       property float x\nproperty float y\nproperty float z\n\
                       end_header\n";
        let mut data = header.to_vec();
        for v in [0.0f32, 0.0, 0.0, 1.0, 1.0, 1.0] {
            data.extend_from_slice(&v.to_le_bytes());
        }
        // Add invalid-UTF-8 bytes to ensure detection doesn't go through from_utf8(full data)
        data.extend_from_slice(&[0xC0, 0xC1, 0xF5]);

        let config = MeshOptConfig::default();
        let result = optimize_ply(&data, &config);
        assert!(result.is_ok(), "binary PLY should not surface as Invalid UTF-8: {:?}", result.err());
    }
}
