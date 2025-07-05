//! Mesh format detection and utilities

use crate::types::{OptError, OptResult};

/// Supported mesh formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    GLTF,
    GLB,
    OBJ,
    STL,
    PLY,
    FBX,
    DAE,
}

impl MeshFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "gltf" => Some(Self::GLTF),
            "glb" => Some(Self::GLB),
            "obj" => Some(Self::OBJ),
            "stl" => Some(Self::STL),
            "ply" => Some(Self::PLY),
            "fbx" => Some(Self::FBX),
            "dae" => Some(Self::DAE),
            _ => None,
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::GLTF => "model/gltf+json",
            Self::GLB => "model/gltf-binary",
            Self::OBJ => "model/obj",
            Self::STL => "application/vnd.ms-pki.stl",
            Self::PLY => "application/ply",
            Self::FBX => "application/octet-stream",
            Self::DAE => "model/vnd.collada+xml",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::GLTF => "gltf",
            Self::GLB => "glb",
            Self::OBJ => "obj",
            Self::STL => "stl",
            Self::PLY => "ply",
            Self::FBX => "fbx",
            Self::DAE => "dae",
        }
    }
}

/// Detect mesh format from file header (magic bytes)
pub fn detect_mesh_format(data: &[u8]) -> OptResult<MeshFormat> {
    if data.len() < 12 {
        return Err(OptError::InvalidFormat("File too small".to_string()));
    }

    // GLB: glTF binary format starts with "glTF"
    if data.starts_with(b"glTF") {
        return Ok(MeshFormat::GLB);
    }

    // STL Binary: starts with 80-byte header
    if data.len() >= 80 && !data[0..80].iter().all(|&b| b.is_ascii()) {
        // Check for STL binary format (not foolproof but reasonable)
        return Ok(MeshFormat::STL);
    }

    // PLY: starts with "ply"
    if data.starts_with(b"ply\n") || data.starts_with(b"ply\r\n") {
        return Ok(MeshFormat::PLY);
    }

    // FBX Binary: starts with "Kaydara FBX Binary"
    if data.starts_with(b"Kaydara FBX Binary") {
        return Ok(MeshFormat::FBX);
    }

    // Text-based format detection (less reliable)
    let text = String::from_utf8_lossy(&data[0..data.len().min(1024)]);
    
    // FBX ASCII: starts with "; FBX" comment
    if text.starts_with("; FBX") {
        return Ok(MeshFormat::FBX);
    }
    
    // GLTF: JSON with gltf-specific properties
    if text.contains("\"asset\"") && text.contains("\"version\"") && text.contains("\"generator\"") {
        return Ok(MeshFormat::GLTF);
    }
    
    // OBJ: starts with comments or vertex definitions
    if text.starts_with('#') || text.contains("v ") || text.contains("f ") {
        return Ok(MeshFormat::OBJ);
    }
    
    // STL ASCII: contains "solid" and "facet"
    if text.to_lowercase().contains("solid") && text.to_lowercase().contains("facet") {
        return Ok(MeshFormat::STL);
    }
    
    // DAE/Collada: XML with COLLADA namespace
    if text.contains("<?xml") && text.contains("COLLADA") {
        return Ok(MeshFormat::DAE);
    }

    Err(OptError::InvalidFormat("Unknown mesh format".to_string()))
}

/// Get basic mesh information without full decode
pub fn get_mesh_info(data: &[u8]) -> OptResult<super::MeshInfo> {
    let format = detect_mesh_format(data)?;
    
    // TODO: Implement proper mesh parsing for accurate vertex/triangle counts
    // For now, provide estimates based on file size
    let estimated_vertices = (data.len() / 100) as u32; // Rough estimate
    let estimated_triangles = estimated_vertices / 3; // Rough estimate

    Ok(super::MeshInfo {
        format,
        vertex_count: estimated_vertices,
        triangle_count: estimated_triangles,
        file_size: data.len(),
    })
}
