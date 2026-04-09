extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}, vec};

use crate::types::{MeshOptConfig, OptResult, OptError};
use crate::mesh::loader::{MeshLoadResult, MeshMetadata, calculate_bounding_box};
use crate::formats::MeshFormat;

const FBX_BINARY_SIGNATURE: &[u8] = b"Kaydara FBX Binary  \x00\x1a\x00";
const FBX_ASCII_SIGNATURE: &str = "; FBX";

pub fn optimize_fbx(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if is_binary_fbx(data) {
        optimize_binary_fbx(data, config)
    } else if is_ascii_fbx(data) {
        let normalized = {
            #[cfg(c_hotspots_available)]
            {
                if !config.preserve_topology && config.target_ratio < 1.0 {
                    if let Some(out) = crate::c_hotspots::util::normalize_text_whitespace_commas(data) {
                        out
                    } else {
                        data.to_vec()
                    }
                } else {
                    data.to_vec()
                }
            }

            #[cfg(not(c_hotspots_available))]
            {
                data.to_vec()
            }
        };

        let content = core::str::from_utf8(&normalized)
            .map_err(|_| OptError::InvalidFormat("FBX ASCII contains invalid UTF-8".to_string()))?;

        let optimized_content = optimize_fbx_ascii(content, config)?;
        Ok(optimized_content.into_bytes())
    } else {
        Err(OptError::InvalidFormat("Not a valid FBX file".to_string()))
    }
}

fn is_binary_fbx(data: &[u8]) -> bool {
    data.len() >= 23 && data.starts_with(FBX_BINARY_SIGNATURE)
}

fn is_ascii_fbx(data: &[u8]) -> bool {
    if data.len() < 10 {
        return false;
    }

    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 200)]) {
        if text.starts_with(FBX_ASCII_SIGNATURE) {
            return true;
        }

        let text_lower = text.to_lowercase();
        return text_lower.contains("fbx") &&
               (text_lower.contains("objects:") ||
                text_lower.contains("fbxheaderextension:") ||
                text_lower.contains("model:") ||
                text_lower.contains("geometry:"));
    }

    false
}

fn optimize_binary_fbx(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if !config.preserve_topology {
        let mut result = data.to_vec();

        while result.last() == Some(&0) {
            result.pop();
        }

        if result.len() < 27 {
            return Ok(data.to_vec());
        }

        Ok(result)
    } else {
        Ok(data.to_vec())
    }
}

fn optimize_fbx_ascii(content: &str, config: &MeshOptConfig) -> OptResult<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut optimized_lines = Vec::<String>::new();

    for line in lines {
        let trimmed = line.trim();

        if config.preserve_topology {
            optimized_lines.push(line.to_string());
            continue;
        }

        if trimmed.starts_with(';') {
            if trimmed.contains("FBX") || trimmed.contains("Creator") {
                optimized_lines.push(line.to_string());
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if config.target_ratio < 1.0 {
            let compressed = compress_fbx_line(line);
            optimized_lines.push(compressed);
        } else {
            optimized_lines.push(line.to_string());
        }
    }

    Ok(optimized_lines.join("\n"))
}

fn compress_fbx_line(line: &str) -> String {
    let trimmed = line.trim();

    if trimmed.contains(',') && (trimmed.contains('.') || trimmed.chars().any(|c| c.is_numeric())) {
        trimmed.split(',').map(|s| s.trim()).collect::<Vec<_>>().join(",")
    } else {
        trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

pub fn validate_fbx_structure(data: &[u8]) -> OptResult<bool> {
    if data.len() < 23 {
        return Err(OptError::InvalidFormat("FBX file too small".to_string()));
    }

    if data.starts_with(FBX_BINARY_SIGNATURE) {
        return Ok(true);
    }

    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 100)]) {
        if text.starts_with(FBX_ASCII_SIGNATURE) {
            let content = core::str::from_utf8(data)
                .map_err(|_| OptError::InvalidFormat("FBX ASCII contains invalid UTF-8".to_string()))?;

            if content.contains("FBX") && (content.contains("Objects:") || content.contains("Model:")) {
                return Ok(true);
            }
        }
    }

    Err(OptError::InvalidFormat("Not a valid FBX file".to_string()))
}

pub fn load_fbx_mesh(data: &[u8]) -> OptResult<MeshLoadResult> {
    if data.len() < 27 || !data.starts_with(FBX_BINARY_SIGNATURE) {
        return load_fbx_ascii(data);
    }

    let version = u32::from_le_bytes([data[23], data[24], data[25], data[26]]);
    let use_u64 = version >= 7500;

    let mut vertices: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut normals: Vec<f32> = Vec::new();
    let mut uvs: Vec<f32> = Vec::new();

    let mut cursor = 27usize;
    walk_fbx_nodes(data, &mut cursor, use_u64, &mut vertices, &mut indices, &mut normals, &mut uvs)?;

    if vertices.is_empty() {
        return Err(OptError::InvalidFormat("FBX binary contains no Vertices node".to_string()));
    }

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
        format: MeshFormat::Fbx,
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

fn walk_fbx_nodes(
    data: &[u8],
    cursor: &mut usize,
    use_u64: bool,
    vertices: &mut Vec<f32>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<f32>,
    uvs: &mut Vec<f32>,
) -> OptResult<()> {
    loop {
        if *cursor >= data.len() {
            return Ok(());
        }
        let (end_offset, num_props, prop_list_len, name_len_pos) = if use_u64 {
            if *cursor + 25 > data.len() {
                return Ok(());
            }
            let eo = u64::from_le_bytes(data[*cursor..*cursor + 8].try_into().unwrap()) as usize;
            let np = u64::from_le_bytes(data[*cursor + 8..*cursor + 16].try_into().unwrap()) as usize;
            let pl = u64::from_le_bytes(data[*cursor + 16..*cursor + 24].try_into().unwrap()) as usize;
            (eo, np, pl, *cursor + 24)
        } else {
            if *cursor + 13 > data.len() {
                return Ok(());
            }
            let eo = u32::from_le_bytes(data[*cursor..*cursor + 4].try_into().unwrap()) as usize;
            let np = u32::from_le_bytes(data[*cursor + 4..*cursor + 8].try_into().unwrap()) as usize;
            let pl = u32::from_le_bytes(data[*cursor + 8..*cursor + 12].try_into().unwrap()) as usize;
            (eo, np, pl, *cursor + 12)
        };

        if end_offset == 0 {
            *cursor = name_len_pos + 1;
            return Ok(());
        }

        if name_len_pos >= data.len() {
            return Ok(());
        }
        let name_len = data[name_len_pos] as usize;
        let name_start = name_len_pos + 1;
        let name_end = name_start + name_len;
        if name_end > data.len() || end_offset > data.len() {
            return Ok(());
        }
        let name = &data[name_start..name_end];

        let props_start = name_end;
        let props_end = props_start + prop_list_len;
        if props_end > data.len() {
            return Ok(());
        }

        let mut prop_cursor = props_start;
        let mut prop_arrays: Vec<(u8, Vec<u8>)> = Vec::new();
        for _ in 0..num_props {
            if prop_cursor >= props_end {
                break;
            }
            let type_code = data[prop_cursor];
            prop_cursor += 1;
            let (consumed, arr_data) = read_fbx_property(data, prop_cursor, type_code, props_end)?;
            prop_cursor += consumed;
            if let Some(bytes) = arr_data {
                prop_arrays.push((type_code, bytes));
            }
        }

        if name == b"Vertices" {
            if let Some((tc, bytes)) = prop_arrays.first() {
                decode_fbx_array_f64(*tc, bytes, vertices);
            }
        } else if name == b"PolygonVertexIndex" {
            if let Some((_, bytes)) = prop_arrays.first() {
                decode_fbx_polygon_indices(bytes, indices);
            }
        } else if name == b"Normals" {
            if let Some((tc, bytes)) = prop_arrays.first() {
                decode_fbx_array_f64(*tc, bytes, normals);
            }
        } else if name == b"UV" {
            if let Some((tc, bytes)) = prop_arrays.first() {
                decode_fbx_array_f64(*tc, bytes, uvs);
            }
        }

        if props_end < end_offset {
            *cursor = props_end;
            while *cursor < end_offset {
                let saved = *cursor;
                walk_fbx_nodes(data, cursor, use_u64, vertices, indices, normals, uvs)?;
                if *cursor <= saved {
                    break;
                }
            }
        }

        *cursor = end_offset;
        if *cursor >= data.len() {
            return Ok(());
        }
    }
}

fn read_fbx_property(data: &[u8], start: usize, type_code: u8, props_end: usize) -> OptResult<(usize, Option<Vec<u8>>)> {
    let scalar_size = match type_code {
        b'Y' => 2,
        b'C' => 1,
        b'I' | b'F' => 4,
        b'D' | b'L' => 8,
        _ => 0,
    };
    if scalar_size > 0 {
        if start + scalar_size > props_end { return Ok((props_end - start, None)); }
        return Ok((scalar_size, None));
    }

    match type_code {
        b'f' | b'i' | b'd' | b'l' | b'b' => {
            if start + 12 > props_end { return Ok((props_end - start, None)); }
            let array_length = u32::from_le_bytes(data[start..start + 4].try_into().unwrap()) as usize;
            let encoding = u32::from_le_bytes(data[start + 4..start + 8].try_into().unwrap());
            let compressed_length = u32::from_le_bytes(data[start + 8..start + 12].try_into().unwrap()) as usize;
            let payload_start = start + 12;
            let payload_end = payload_start + compressed_length;
            if payload_end > props_end { return Ok((props_end - start, None)); }
            if encoding == 0 {
                let elem_size = match type_code {
                    b'f' | b'i' => 4,
                    b'd' | b'l' => 8,
                    b'b' => 1,
                    _ => 4,
                };
                let expected = array_length * elem_size;
                let take = expected.min(compressed_length);
                let mut bytes = Vec::with_capacity(take + 1);
                bytes.push(type_code);
                bytes.extend_from_slice(&data[payload_start..payload_start + take]);
                Ok((12 + compressed_length, Some(bytes)))
            } else {
                let inflated = inflate_zlib(&data[payload_start..payload_end]);
                let bytes = inflated.map(|b| {
                    let mut tagged = Vec::with_capacity(b.len() + 1);
                    tagged.push(type_code);
                    tagged.extend_from_slice(&b);
                    tagged
                });
                Ok((12 + compressed_length, bytes))
            }
        }
        b'S' | b'R' => {
            if start + 4 > props_end { return Ok((props_end - start, None)); }
            let length = u32::from_le_bytes(data[start..start + 4].try_into().unwrap()) as usize;
            let total = 4 + length;
            if start + total > props_end { return Ok((props_end - start, None)); }
            Ok((total, None))
        }
        _ => Ok((props_end - start, None)),
    }
}

fn decode_fbx_array_f64(_type_code: u8, payload: &[u8], out: &mut Vec<f32>) {
    if payload.is_empty() {
        return;
    }
    let inner_type = payload[0];
    let bytes = &payload[1..];
    match inner_type {
        b'd' => {
            for chunk in bytes.chunks_exact(8) {
                let v = f64::from_le_bytes(chunk.try_into().unwrap());
                out.push(v as f32);
            }
        }
        b'f' => {
            for chunk in bytes.chunks_exact(4) {
                let v = f32::from_le_bytes(chunk.try_into().unwrap());
                out.push(v);
            }
        }
        _ => {}
    }
}

fn decode_fbx_polygon_indices(payload: &[u8], indices: &mut Vec<u32>) {
    if payload.is_empty() {
        return;
    }
    let inner_type = payload[0];
    let bytes = &payload[1..];
    let mut polygon: Vec<u32> = Vec::new();

    let mut process = |raw: i32| {
        let end_of_polygon = raw < 0;
        let actual = if end_of_polygon { (!raw) as u32 } else { raw as u32 };
        polygon.push(actual);
        if end_of_polygon {
            if polygon.len() >= 3 {
                for i in 1..polygon.len() - 1 {
                    indices.push(polygon[0]);
                    indices.push(polygon[i]);
                    indices.push(polygon[i + 1]);
                }
            }
            polygon.clear();
        }
    };

    match inner_type {
        b'i' => {
            for chunk in bytes.chunks_exact(4) {
                process(i32::from_le_bytes(chunk.try_into().unwrap()));
            }
        }
        b'l' => {
            for chunk in bytes.chunks_exact(8) {
                process(i64::from_le_bytes(chunk.try_into().unwrap()) as i32);
            }
        }
        _ => {}
    }
}

fn inflate_zlib(_input: &[u8]) -> Option<Vec<u8>> {
    #[cfg(feature = "compression")]
    {
        use flate2::Decompress;
        use flate2::FlushDecompress;
        let mut decoder = Decompress::new(true);
        let mut out: Vec<u8> = Vec::with_capacity(_input.len() * 4);
        let mut buffer = vec![0u8; 64 * 1024];
        let mut consumed = 0usize;
        loop {
            let before_in = decoder.total_in();
            let before_out = decoder.total_out();
            let status = decoder
                .decompress(&_input[consumed..], &mut buffer, FlushDecompress::None)
                .ok()?;
            let read = (decoder.total_in() - before_in) as usize;
            let written = (decoder.total_out() - before_out) as usize;
            consumed += read;
            out.extend_from_slice(&buffer[..written]);
            match status {
                flate2::Status::StreamEnd => return Some(out),
                flate2::Status::Ok | flate2::Status::BufError => {
                    if read == 0 && written == 0 {
                        return None;
                    }
                }
            }
            if consumed >= _input.len() {
                return Some(out);
            }
        }
    }
    #[cfg(not(feature = "compression"))]
    {
        None
    }
}

fn load_fbx_ascii(data: &[u8]) -> OptResult<MeshLoadResult> {
    let content = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("FBX ASCII contains invalid UTF-8".to_string()))?;

    let vertices = extract_fbx_ascii_array(content, "Vertices:");
    let raw_indices = extract_fbx_ascii_int_array(content, "PolygonVertexIndex:");
    let normals = extract_fbx_ascii_array(content, "Normals:");
    let uvs = extract_fbx_ascii_array(content, "UV:");

    let mut indices: Vec<u32> = Vec::new();
    let mut polygon: Vec<u32> = Vec::new();
    for raw in raw_indices {
        let end_of_polygon = raw < 0;
        let actual = if end_of_polygon { (!raw) as u32 } else { raw as u32 };
        polygon.push(actual);
        if end_of_polygon {
            if polygon.len() >= 3 {
                for i in 1..polygon.len() - 1 {
                    indices.push(polygon[0]);
                    indices.push(polygon[i]);
                    indices.push(polygon[i + 1]);
                }
            }
            polygon.clear();
        }
    }

    if vertices.is_empty() {
        return Err(OptError::InvalidFormat("FBX ASCII contains no Vertices array".to_string()));
    }

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
        format: MeshFormat::Fbx,
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

fn extract_fbx_ascii_array(content: &str, key: &str) -> Vec<f32> {
    let idx = match content.find(key) {
        Some(i) => i,
        None => return Vec::new(),
    };
    let after_key = &content[idx + key.len()..];
    let array_start = match after_key.find("a:") {
        Some(i) => i + 2,
        None => return Vec::new(),
    };
    let array_end = match after_key[array_start..].find('}') {
        Some(i) => array_start + i,
        None => after_key.len(),
    };
    let slice = &after_key[array_start..array_end];
    slice
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
        .collect()
}

fn extract_fbx_ascii_int_array(content: &str, key: &str) -> Vec<i32> {
    let idx = match content.find(key) {
        Some(i) => i,
        None => return Vec::new(),
    };
    let after_key = &content[idx + key.len()..];
    let array_start = match after_key.find("a:") {
        Some(i) => i + 2,
        None => return Vec::new(),
    };
    let array_end = match after_key[array_start..].find('}') {
        Some(i) => array_start + i,
        None => after_key.len(),
    };
    let slice = &after_key[array_start..array_end];
    slice
        .split(',')
        .filter_map(|s| s.trim().parse::<i32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fbx_signature_detection() {
        let binary_fbx_header = [
            b"Kaydara FBX Binary  \x00\x1a\x00".to_vec(),
            alloc::vec![0x00, 0x00],
            alloc::vec![0xE8, 0x1C, 0x00, 0x00],
        ].concat();

        assert!(binary_fbx_header.starts_with(FBX_BINARY_SIGNATURE));

        let ascii_fbx = b"; FBX 7.4.0 project file";
        assert!(ascii_fbx.starts_with(FBX_ASCII_SIGNATURE.as_bytes()));
    }

    #[test]
    fn test_fbx_validation() {
        let ascii_fbx = b"; FBX 7.4.0 project file
; Created by test
FBXHeaderExtension:  {
    FBXHeaderVersion: 1003
}
Objects:  {
    Model: 1234, \"Cube\", \"Mesh\" {
    }
}
";

        let result = validate_fbx_structure(ascii_fbx);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let invalid_fbx = b"not_an_fbx_file";
        let result = validate_fbx_structure(invalid_fbx);
        assert!(result.is_err());
    }

    #[test]
    fn test_fbx_ascii_array_extraction() {
        let content = "Vertices: *3 {\n  a: 1.0,2.0,3.0\n}";
        let vertices = extract_fbx_ascii_array(content, "Vertices:");
        assert_eq!(vertices, alloc::vec![1.0, 2.0, 3.0]);
    }
}
