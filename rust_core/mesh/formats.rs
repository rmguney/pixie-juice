//! Mesh format detection and utilities

use crate::types::{OptError, OptResult};

/// Quick parsing functions for getting vertex/triangle counts

fn parse_obj_counts(data: &[u8]) -> OptResult<(u32, u32)> {
    let content = std::str::from_utf8(data)
        .map_err(|_| OptError::ProcessingError("Invalid UTF-8 in OBJ file".to_string()))?;
    
    let mut vertex_count = 0;
    let mut face_count = 0;
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("v ") {
            vertex_count += 1;
        } else if line.starts_with("f ") {
            face_count += 1;
        }
    }
    
    // Assume triangulated faces for triangle count
    Ok((vertex_count, face_count))
}

fn parse_ply_counts(data: &[u8]) -> OptResult<(u32, u32)> {
    let header_end = data.windows(10).position(|w| w == b"end_header")
        .ok_or_else(|| OptError::ProcessingError("Invalid PLY file: no end_header found".to_string()))?;
    
    let header = std::str::from_utf8(&data[..header_end])
        .map_err(|_| OptError::ProcessingError("Invalid UTF-8 in PLY header".to_string()))?;
    
    let mut vertex_count = 0;
    let mut face_count = 0;
    
    for line in header.lines() {
        if line.starts_with("element vertex ") {
            vertex_count = line.split_whitespace().nth(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        } else if line.starts_with("element face ") {
            face_count = line.split_whitespace().nth(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
        }
    }
    
    Ok((vertex_count, face_count))
}

fn parse_stl_counts(data: &[u8]) -> OptResult<(u32, u32)> {
    if data.len() < 80 {
        return Err(OptError::ProcessingError("STL file too small".to_string()));
    }
    
    // Check if it's binary STL (first 5 bytes != "solid")
    if !data.starts_with(b"solid") || data.len() < 84 {
        // Binary STL: triangle count is at bytes 80-83
        if data.len() >= 84 {
            let triangle_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]);
            let vertex_count = triangle_count * 3; // Each triangle has 3 vertices
            Ok((vertex_count, triangle_count))
        } else {
            Err(OptError::ProcessingError("Invalid binary STL file".to_string()))
        }
    } else {
        // ASCII STL: count "facet normal" occurrences
        let content = std::str::from_utf8(data)
            .map_err(|_| OptError::ProcessingError("Invalid UTF-8 in ASCII STL file".to_string()))?;
        
        let triangle_count = content.matches("facet normal").count() as u32;
        let vertex_count = triangle_count * 3;
        Ok((vertex_count, triangle_count))
    }
}

fn parse_gltf_counts(data: &[u8]) -> OptResult<(u32, u32)> {
    // For GLTF/GLB, this would require proper JSON/binary parsing
    // For now, provide rough estimates based on file size
    let estimated_vertices = (data.len() / 50) as u32; // GLTF is more compact than OBJ
    let estimated_triangles = estimated_vertices / 3;
    Ok((estimated_vertices, estimated_triangles))
}

fn parse_fbx_counts(data: &[u8]) -> OptResult<(u32, u32)> {
    // FBX is a complex binary format, provide rough estimates
    let estimated_vertices = (data.len() / 60) as u32; 
    let estimated_triangles = estimated_vertices / 3;
    Ok((estimated_vertices, estimated_triangles))
}

fn parse_dae_counts(data: &[u8]) -> OptResult<(u32, u32)> {
    // DAE is XML-based, could parse for <vertices> and <triangles> elements
    // For now, provide rough estimates
    let estimated_vertices = (data.len() / 80) as u32; // XML is verbose
    let estimated_triangles = estimated_vertices / 3;
    Ok((estimated_vertices, estimated_triangles))
}

fn parse_usdz_counts(data: &[u8]) -> OptResult<(u32, u32)> {
    // USDZ is a ZIP archive containing USD files
    // For now, provide rough estimates based on file size
    let estimated_vertices = (data.len() / 70) as u32; // USD is relatively compact
    let estimated_triangles = estimated_vertices / 3;
    Ok((estimated_vertices, estimated_triangles))
}

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
    USDZ,
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
            "usdz" => Some(Self::USDZ),
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
            Self::USDZ => "model/vnd.usdz+zip",
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
            Self::USDZ => "usdz",
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

    // USDZ: ZIP archive (starts with ZIP signature)
    if data.starts_with(b"PK\x03\x04") || data.starts_with(b"PK\x05\x06") {
        // This is a ZIP file, could be USDZ (need to check extension or content)
        return Ok(MeshFormat::USDZ);
    }

    // Text-based format detection (less reliable)
    let text = String::from_utf8_lossy(&data[0..data.len().min(1024)]);
    
    // glTF JSON: contains "asset" and "version" keys
    if text.contains("\"asset\"") && text.contains("\"version\"") {
        return Ok(MeshFormat::GLTF);
    }
    
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
    
    // Parse mesh to get accurate vertex and triangle counts
    let (vertex_count, triangle_count) = match format {
        MeshFormat::OBJ => parse_obj_counts(data)?,
        MeshFormat::PLY => parse_ply_counts(data)?,
        MeshFormat::STL => parse_stl_counts(data)?,
        MeshFormat::GLTF | MeshFormat::GLB => parse_gltf_counts(data)?,
        MeshFormat::FBX => parse_fbx_counts(data)?,
        MeshFormat::DAE => parse_dae_counts(data)?,
        MeshFormat::USDZ => parse_usdz_counts(data)?,
    };

    Ok(super::MeshInfo {
        format,
        vertex_count,
        triangle_count,
        file_size: data.len(),
    })
}
