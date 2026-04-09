extern crate alloc;
use alloc::{vec::Vec, string::{String, ToString}, format};
use crate::types::{OptResult, MeshOptConfig, OptError};

const GLB_MAGIC: &[u8; 4] = b"glTF";
const CHUNK_TYPE_JSON: u32 = 0x4E4F_534A;
const CHUNK_TYPE_BIN: u32 = 0x004E_4942;

pub fn optimize_gltf(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if data.len() < 20 {
        return Err(OptError::InvalidFormat("glTF file too small".to_string()));
    }

    if data.len() >= 4 && &data[0..4] == GLB_MAGIC {
        optimize_glb_chunks(data, config)
    } else {
        optimize_json_gltf(data, config)
    }
}

pub fn optimize_glb(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    optimize_glb_chunks(data, config)
}

fn optimize_json_gltf(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    let text = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("glTF JSON is not valid UTF-8".to_string()))?;

    let mut value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| OptError::InvalidFormat(format!("glTF JSON parse failed: {}", e)))?;

    let aggressive = config.target_ratio < 1.0;
    strip_gltf_json(&mut value, aggressive);

    let bytes = serde_json::to_vec(&value)
        .map_err(|e| OptError::ProcessingError(format!("glTF JSON serialize failed: {}", e)))?;

    if bytes.len() < data.len() {
        Ok(bytes)
    } else {
        Ok(data.to_vec())
    }
}

fn optimize_glb_chunks(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if data.len() < 12 || &data[0..4] != GLB_MAGIC {
        return Err(OptError::InvalidFormat("Invalid GLB magic".to_string()));
    }

    let _version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let total_length = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    if total_length > data.len() {
        return Err(OptError::InvalidFormat("GLB declared length exceeds file size".to_string()));
    }

    let mut cursor = 12usize;
    let mut json_chunk: Option<&[u8]> = None;
    let mut bin_chunk: Option<&[u8]> = None;
    while cursor + 8 <= total_length {
        let chunk_len = u32::from_le_bytes([data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3]]) as usize;
        let chunk_type = u32::from_le_bytes([data[cursor + 4], data[cursor + 5], data[cursor + 6], data[cursor + 7]]);
        let body_start = cursor + 8;
        let body_end = body_start.checked_add(chunk_len)
            .ok_or_else(|| OptError::InvalidFormat("GLB chunk length overflow".to_string()))?;
        if body_end > total_length {
            return Err(OptError::InvalidFormat("GLB chunk extends past file".to_string()));
        }
        match chunk_type {
            CHUNK_TYPE_JSON => json_chunk = Some(&data[body_start..body_end]),
            CHUNK_TYPE_BIN => bin_chunk = Some(&data[body_start..body_end]),
            _ => {}
        }
        cursor = body_end;
    }

    let json_bytes = json_chunk
        .ok_or_else(|| OptError::InvalidFormat("GLB missing JSON chunk".to_string()))?;

    let json_text = core::str::from_utf8(json_bytes)
        .map_err(|_| OptError::InvalidFormat("GLB JSON chunk not valid UTF-8".to_string()))?
        .trim_end_matches('\0')
        .trim_end_matches(' ');

    let mut value: serde_json::Value = serde_json::from_str(json_text)
        .map_err(|e| OptError::InvalidFormat(format!("GLB JSON parse failed: {}", e)))?;

    let aggressive = config.target_ratio < 1.0;
    strip_gltf_json(&mut value, aggressive);

    let compact_json = serde_json::to_vec(&value)
        .map_err(|e| OptError::ProcessingError(format!("GLB JSON serialize failed: {}", e)))?;

    let padded_json = pad_chunk(compact_json, b' ');
    let padded_bin = bin_chunk.map(|b| pad_chunk(b.to_vec(), 0));

    let mut new_total: usize = 12 + 8 + padded_json.len();
    if let Some(ref b) = padded_bin {
        new_total += 8 + b.len();
    }

    let mut out = Vec::with_capacity(new_total);
    out.extend_from_slice(GLB_MAGIC);
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(new_total as u32).to_le_bytes());

    out.extend_from_slice(&(padded_json.len() as u32).to_le_bytes());
    out.extend_from_slice(&CHUNK_TYPE_JSON.to_le_bytes());
    out.extend_from_slice(&padded_json);

    if let Some(b) = padded_bin {
        out.extend_from_slice(&(b.len() as u32).to_le_bytes());
        out.extend_from_slice(&CHUNK_TYPE_BIN.to_le_bytes());
        out.extend_from_slice(&b);
    }

    if out.len() < data.len() {
        Ok(out)
    } else {
        Ok(data.to_vec())
    }
}

fn pad_chunk(mut bytes: Vec<u8>, pad_byte: u8) -> Vec<u8> {
    let remainder = bytes.len() % 4;
    if remainder != 0 {
        let pad = 4 - remainder;
        bytes.extend(core::iter::repeat(pad_byte).take(pad));
    }
    bytes
}

fn strip_gltf_json(value: &mut serde_json::Value, aggressive: bool) {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            if aggressive {
                map.remove("extras");
            }
            let mut to_remove: Vec<String> = Vec::new();
            for (k, v) in map.iter() {
                if value_is_empty(v) {
                    to_remove.push(k.clone());
                }
                if aggressive && (k == "name" || k == "copyright" || k == "generator") {
                    to_remove.push(k.clone());
                }
            }
            for k in to_remove {
                map.remove(&k);
            }
            for (_, v) in map.iter_mut() {
                strip_gltf_json(v, aggressive);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                strip_gltf_json(item, aggressive);
            }
        }
        _ => {}
    }
}

fn value_is_empty(v: &serde_json::Value) -> bool {
    use serde_json::Value;
    match v {
        Value::Null => true,
        Value::String(s) => s.is_empty(),
        Value::Array(a) => a.is_empty(),
        Value::Object(o) => o.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_gltf_compaction() {
        let pretty = br#"{
    "asset": {
        "version": "2.0",
        "generator": "test"
    },
    "scenes": [],
    "nodes": []
}"#;
        let config = MeshOptConfig::default();
        let result = optimize_gltf(pretty, &config).unwrap();
        let s = core::str::from_utf8(&result).unwrap();
        assert!(s.contains("\"asset\""));
        assert!(!s.contains("    "));
    }

    #[test]
    fn test_glb_header_preserved() {
        let json = br#"{"asset":{"version":"2.0"}}"#;
        let mut json_padded = json.to_vec();
        while json_padded.len() % 4 != 0 {
            json_padded.push(b' ');
        }
        let json_len = json_padded.len() as u32;

        let mut bin: Vec<u8> = (0u8..16u8).collect();
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let bin_len = bin.len() as u32;

        let total: u32 = 12 + 8 + json_len + 8 + bin_len;

        let mut glb: Vec<u8> = Vec::new();
        glb.extend_from_slice(GLB_MAGIC);
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&total.to_le_bytes());
        glb.extend_from_slice(&json_len.to_le_bytes());
        glb.extend_from_slice(&CHUNK_TYPE_JSON.to_le_bytes());
        glb.extend_from_slice(&json_padded);
        glb.extend_from_slice(&bin_len.to_le_bytes());
        glb.extend_from_slice(&CHUNK_TYPE_BIN.to_le_bytes());
        glb.extend_from_slice(&bin);

        let config = MeshOptConfig::default();
        let out = optimize_glb(&glb, &config).unwrap();
        assert_eq!(&out[0..4], GLB_MAGIC);
        let out_total = u32::from_le_bytes([out[8], out[9], out[10], out[11]]) as usize;
        assert_eq!(out_total, out.len());
    }
}
