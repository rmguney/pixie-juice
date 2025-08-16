//! Mesh format detection and metadata

extern crate alloc;
use alloc::{vec, vec::Vec, string::{String, ToString}, format};
use crate::types::{OptResult, OptError};
use crate::formats::MeshFormat;

/// Detect mesh format from file data using proven format signatures
pub fn detect_mesh_format(data: &[u8]) -> OptResult<MeshFormat> {
    if data.len() < 4 {
        return Err(OptError::InvalidFormat("File too small to determine format".to_string()));
    }
    
    // OBJ format detection (ASCII text-based)
    if is_obj_format(data) {
        return Ok(MeshFormat::Obj);
    }
    
    // glTF format detection (JSON-based)
    if is_gltf_format(data) {
        return Ok(MeshFormat::Gltf);
    }
    
    // GLB format detection (binary glTF)
    if is_glb_format(data) {
        return Ok(MeshFormat::Glb);
    }
    
    // STL format detection (both ASCII and binary)
    if is_stl_format(data) {
        return Ok(MeshFormat::Stl);
    }
    
    // PLY format detection
    if is_ply_format(data) {
        return Ok(MeshFormat::Ply);
    }
    
    // FBX format detection (complex binary format)
    if is_fbx_format(data) {
        return Ok(MeshFormat::Fbx);
    }
    
    Err(OptError::InvalidFormat("Unknown mesh format".to_string()))
}

/// Detect OBJ format using proven text-based signatures
fn is_obj_format(data: &[u8]) -> bool {
    if let Ok(text) = core::str::from_utf8(data) {
        let lines: Vec<&str> = text.lines().take(10).collect();
        
        // Check for common OBJ keywords in first few lines
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") || 
               trimmed.starts_with("v ") || 
               trimmed.starts_with("vn ") || 
               trimmed.starts_with("vt ") || 
               trimmed.starts_with("f ") ||
               trimmed.starts_with("o ") ||
               trimmed.starts_with("g ") {
                return true;
            }
        }
    }
    false
}

/// Detect glTF format using JSON structure validation
fn is_gltf_format(data: &[u8]) -> bool {
    if let Ok(text) = core::str::from_utf8(data) {
        // Check for glTF JSON signature
        if text.trim_start().starts_with('{') {
            // Look for required glTF fields
            return text.contains("\"asset\"") && 
                   text.contains("\"version\"") &&
                   (text.contains("\"meshes\"") || text.contains("\"scenes\""));
        }
    }
    false
}

/// Detect GLB (binary glTF) format using magic number
fn is_glb_format(data: &[u8]) -> bool {
    data.len() >= 12 && 
    &data[0..4] == b"glTF" // GLB magic number
}

/// Detect STL format (both ASCII and binary)
fn is_stl_format(data: &[u8]) -> bool {
    // ASCII STL detection
    if let Ok(text) = core::str::from_utf8(data) {
        let first_line = text.lines().next().unwrap_or("");
        if first_line.trim().to_lowercase().starts_with("solid") {
            return true;
        }
    }
    
    // Binary STL detection (80-byte header + 4-byte triangle count)
    if data.len() >= 84 {
        // Get triangle count from bytes 80-83
        let triangle_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]);
        // Each triangle is 50 bytes (12 floats + 2 bytes attributes)
        let expected_size = 84 + (triangle_count as usize * 50);
        
        // Allow some tolerance in file size
        return data.len() >= expected_size && data.len() <= expected_size + 100;
    }
    
    false
}

/// Detect PLY format using proven signature
fn is_ply_format(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    
    // PLY files start with "ply" followed by newline
    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 100)]) {
        let first_line = text.lines().next().unwrap_or("");
        return first_line.trim().eq_ignore_ascii_case("ply");
    }
    
    false
}

/// Detect FBX format using binary signature
fn is_fbx_format(data: &[u8]) -> bool {
    if data.len() < 23 {
        return false;
    }
    
    // Binary FBX signature
    const FBX_BINARY_SIGNATURE: &[u8] = b"Kaydara FBX Binary  \x00\x1a\x00";
    if data.starts_with(FBX_BINARY_SIGNATURE) {
        return true;
    }
    
    // ASCII FBX signature  
    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 100)]) {
        if text.contains("; FBX ") || text.contains("FBX version") {
            return true;
        }
    }
    
    false
}

/// Get mesh format metadata and capabilities
pub fn get_mesh_format_info(format: &MeshFormat) -> MeshFormatInfo {
    match format {
        MeshFormat::Obj => MeshFormatInfo {
            name: "Wavefront OBJ".to_string(),
            extensions: vec!["obj".to_string()],
            supports_textures: true,
            supports_materials: true,
            supports_animations: false,
            typical_compression_ratio: 0.85, // Text-based, moderate compression
            optimization_strategies: vec![
                "Vertex welding".to_string(),
                "Normal recalculation".to_string(),
                "Mesh decimation".to_string(),
            ],
        },
        MeshFormat::Gltf => MeshFormatInfo {
            name: "GL Transmission Format".to_string(),
            extensions: vec!["gltf".to_string()],
            supports_textures: true,
            supports_materials: true,
            supports_animations: true,
            typical_compression_ratio: 0.75, // JSON-based, good compression potential
            optimization_strategies: vec![
                "JSON minification".to_string(),
                "Buffer optimization".to_string(),
                "Mesh primitive merging".to_string(),
            ],
        },
        MeshFormat::Glb => MeshFormatInfo {
            name: "GL Transmission Format Binary".to_string(),
            extensions: vec!["glb".to_string()],
            supports_textures: true,
            supports_materials: true,
            supports_animations: true,
            typical_compression_ratio: 0.95, // Already binary, limited compression
            optimization_strategies: vec![
                "Conservative optimization".to_string(),
                "Metadata stripping".to_string(),
            ],
        },
        MeshFormat::Stl => MeshFormatInfo {
            name: "Stereolithography".to_string(),
            extensions: vec!["stl".to_string()],
            supports_textures: false,
            supports_materials: false,
            supports_animations: false,
            typical_compression_ratio: 0.70, // Triangle-based, good decimation potential
            optimization_strategies: vec![
                "Mesh decimation".to_string(),
                "Normal optimization".to_string(),
                "Binary conversion".to_string(),
            ],
        },
        MeshFormat::Ply => MeshFormatInfo {
            name: "Stanford Polygon Format".to_string(),
            extensions: vec!["ply".to_string()],
            supports_textures: false,
            supports_materials: true,
            supports_animations: false,
            typical_compression_ratio: 0.80, // Flexible format, moderate compression
            optimization_strategies: vec![
                "Property optimization".to_string(),
                "Binary conversion".to_string(),
                "Vertex welding".to_string(),
            ],
        },
        MeshFormat::Fbx => MeshFormatInfo {
            name: "Autodesk Filmbox".to_string(),
            extensions: vec!["fbx".to_string()],
            supports_textures: true,
            supports_materials: true,
            supports_animations: true,
            typical_compression_ratio: 0.90, // Complex binary format, limited optimization
            optimization_strategies: vec![
                "Conservative optimization".to_string(),
                "Metadata cleanup".to_string(),
            ],
        },
    }
}

/// Mesh format capabilities and metadata
#[derive(Debug, Clone)]
pub struct MeshFormatInfo {
    pub name: String,
    pub extensions: Vec<String>,
    pub supports_textures: bool,
    pub supports_materials: bool,
    pub supports_animations: bool,
    pub typical_compression_ratio: f32,
    pub optimization_strategies: Vec<String>,
}

/// Validate mesh data integrity
pub fn validate_mesh_data(data: &[u8], format: &MeshFormat) -> OptResult<()> {
    match format {
        MeshFormat::Obj => validate_obj_data(data),
        MeshFormat::Gltf => validate_gltf_data(data),
        MeshFormat::Glb => validate_glb_data(data),
        MeshFormat::Stl => validate_stl_data(data),
        MeshFormat::Ply => validate_ply_data(data),
        MeshFormat::Fbx => validate_fbx_data(data),
    }
}

/// Validate OBJ format integrity
fn validate_obj_data(data: &[u8]) -> OptResult<()> {
    let text = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("OBJ file contains invalid UTF-8".to_string()))?;
    
    let mut has_vertices = false;
    let mut has_faces = false;
    
    for line in text.lines().take(1000) { // Check first 1000 lines for performance
        let trimmed = line.trim();
        if trimmed.starts_with("v ") {
            has_vertices = true;
        } else if trimmed.starts_with("f ") {
            has_faces = true;
        }
        
        if has_vertices && has_faces {
            return Ok(());
        }
    }
    
    if !has_vertices {
        return Err(OptError::InvalidFormat("OBJ file missing vertices".to_string()));
    }
    
    Ok(())
}

/// Validate glTF JSON format
fn validate_gltf_data(data: &[u8]) -> OptResult<()> {
    let text = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("glTF file contains invalid UTF-8".to_string()))?;
    
    // Basic JSON validation
    if !text.trim().starts_with('{') || !text.trim().ends_with('}') {
        return Err(OptError::InvalidFormat("Invalid glTF JSON structure".to_string()));
    }
    
    // Check for required glTF fields
    if !text.contains("\"asset\"") {
        return Err(OptError::InvalidFormat("glTF missing required 'asset' field".to_string()));
    }
    
    Ok(())
}

/// Validate GLB binary format
fn validate_glb_data(data: &[u8]) -> OptResult<()> {
    if data.len() < 20 {
        return Err(OptError::InvalidFormat("GLB file too small".to_string()));
    }
    
    // Check GLB header structure
    if &data[0..4] != b"glTF" {
        return Err(OptError::InvalidFormat("Invalid GLB magic number".to_string()));
    }
    
    // Check version (bytes 4-7)
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != 2 {
        return Err(OptError::InvalidFormat(format!("Unsupported GLB version: {}", version)));
    }
    
    Ok(())
}

/// Validate STL format
fn validate_stl_data(data: &[u8]) -> OptResult<()> {
    if data.len() < 84 {
        return Err(OptError::InvalidFormat("STL file too small".to_string()));
    }
    
    // Check if it's ASCII STL
    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 100)]) {
        if text.trim().to_lowercase().starts_with("solid") {
            // ASCII STL validation
            if !text.to_lowercase().contains("endsolid") {
                return Err(OptError::InvalidFormat("ASCII STL missing 'endsolid'".to_string()));
            }
            return Ok(());
        }
    }
    
    // Binary STL validation
    let triangle_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]);
    let expected_size = 84 + (triangle_count as usize * 50);
    
    if data.len() < expected_size {
        return Err(OptError::InvalidFormat("Binary STL file size mismatch".to_string()));
    }
    
    Ok(())
}

/// Validate PLY format
fn validate_ply_data(data: &[u8]) -> OptResult<()> {
    let text = core::str::from_utf8(data)
        .map_err(|_| OptError::InvalidFormat("PLY file contains invalid UTF-8".to_string()))?;
    
    let mut lines = text.lines();
    
    // First line must be "ply"
    if let Some(first_line) = lines.next() {
        if !first_line.trim().eq_ignore_ascii_case("ply") {
            return Err(OptError::InvalidFormat("PLY file missing 'ply' header".to_string()));
        }
    } else {
        return Err(OptError::InvalidFormat("Empty PLY file".to_string()));
    }
    
    // Check for required format line
    let mut has_format = false;
    for line in lines.take(10) {
        if line.trim().starts_with("format ") {
            has_format = true;
            break;
        }
    }
    
    if !has_format {
        return Err(OptError::InvalidFormat("PLY file missing format specification".to_string()));
    }
    
    Ok(())
}

/// Validate FBX format (basic validation)
fn validate_fbx_data(data: &[u8]) -> OptResult<()> {
    if data.len() < 23 {
        return Err(OptError::InvalidFormat("FBX file too small".to_string()));
    }
    
    // Check binary FBX signature
    const FBX_BINARY_SIGNATURE: &[u8] = b"Kaydara FBX Binary  \x00\x1a\x00";
    if data.starts_with(FBX_BINARY_SIGNATURE) {
        return Ok(()); // Binary FBX is valid
    }
    
    // Check ASCII FBX
    if let Ok(text) = core::str::from_utf8(&data[0..core::cmp::min(data.len(), 200)]) {
        if text.contains("; FBX ") {
            return Ok(());
        }
    }
    
    Err(OptError::InvalidFormat("Invalid FBX format".to_string()))
}
