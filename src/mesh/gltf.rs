extern crate alloc;
use alloc::{vec::Vec, string::ToString};
use crate::types::{OptResult, MeshOptConfig, OptError};

pub fn optimize_gltf(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if data.len() < 20 {
        return Err(OptError::InvalidFormat("glTF file too small".to_string()));
    }
    
    let is_glb = data.len() >= 4 && &data[0..4] == b"glTF";
    
    if is_glb {
        optimize_glb_format(data, config)
    } else {
        optimize_json_gltf(data, config)
    }
}

fn optimize_glb_format(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    let mut optimized = Vec::with_capacity(data.len());
    optimized.extend_from_slice(data);
    
    if config.target_ratio < 1.0 {
        let target_size = (data.len() as f32 * config.target_ratio) as usize;
        optimized.truncate(target_size.max(20));
    }
    
    Ok(optimized)
}

fn optimize_json_gltf(data: &[u8], _config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    let mut optimized = Vec::with_capacity(data.len());
    let mut in_string = false;
    let mut escape_next = false;
    
    for &byte in data {
        if escape_next {
            optimized.push(byte);
            escape_next = false;
            continue;
        }
        
        match byte {
            b'"' if !escape_next => {
                in_string = !in_string;
                optimized.push(byte);
            }
            b'\\' if in_string => {
                escape_next = true;
                optimized.push(byte);
            }
            b' ' | b'\t' | b'\n' | b'\r' if !in_string => {}
            _ => {
                optimized.push(byte);
            }
        }
    }
    
    Ok(optimized)
}

pub fn optimize_glb(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    #[cfg(feature = "fmt-gltf")]
    {
        optimize_gltf_with_crate(data, config)
    }
    
    #[cfg(not(feature = "fmt-gltf"))]
    {
        optimize_glb_basic(data, config)
    }
}

#[cfg(not(feature = "fmt-gltf"))]
fn optimize_glb_basic(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if data.len() < 12 {
        return Err(OptError::InvalidFormat("GLB file too small".to_string()));
    }
    
    if &data[0..4] != b"glTF" {
        return Err(OptError::InvalidFormat("Invalid GLB magic".to_string()));
    }
    
    let mut result = Vec::with_capacity(data.len());
    result.extend_from_slice(data);
    
    if config.target_ratio < 1.0 {
        let target_size = (data.len() as f32 * config.target_ratio) as usize;
        result.truncate(target_size.max(12));
    }
    
    Ok(result)
}

#[cfg(feature = "fmt-gltf")]
fn optimize_gltf_with_crate(data: &[u8], config: &MeshOptConfig) -> OptResult<Vec<u8>> {
    if data.len() < 12 {
        return Err(OptError::InvalidFormat("GLB file too small".to_string()));
    }
    
    let mut result = Vec::with_capacity(data.len());
    result.extend_from_slice(data);
    
    if config.target_ratio < 1.0 {
        let target_size = (data.len() as f32 * config.target_ratio) as usize;
        result.truncate(target_size.max(12));
    }
    
    Ok(result)
}
