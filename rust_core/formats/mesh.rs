//! 3D mesh format handling

use crate::{OptError, OptResult};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    Obj,
    Ply,
    Stl,
    Gltf,
    Glb,
}

impl MeshFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> OptResult<Self> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| OptError::FormatError("No file extension".to_string()))?;
        
        match ext.to_lowercase().as_str() {
            "obj" => Ok(Self::Obj),
            "ply" => Ok(Self::Ply),
            "stl" => Ok(Self::Stl),
            "gltf" => Ok(Self::Gltf),
            "glb" => Ok(Self::Glb),
            _ => Err(OptError::FormatError(format!("Unsupported mesh format: {}", ext))),
        }
    }
    
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Obj => "obj",
            Self::Ply => "ply",
            Self::Stl => "stl",
            Self::Gltf => "gltf",
            Self::Glb => "glb",
        }
    }
    
    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Obj => "model/obj",
            Self::Ply => "model/ply",
            Self::Stl => "model/stl",
            Self::Gltf => "model/gltf+json",
            Self::Glb => "model/gltf-binary",
        }
    }
    
    /// Check if format supports binary encoding
    pub fn supports_binary(&self) -> bool {
        matches!(self, Self::Ply | Self::Stl | Self::Glb)
    }
    
    /// Check if format supports materials/textures
    pub fn supports_materials(&self) -> bool {
        matches!(self, Self::Obj | Self::Gltf | Self::Glb)
    }
    
    /// Check if format is suitable for web use
    pub fn web_compatible(&self) -> bool {
        matches!(self, Self::Gltf | Self::Glb)
    }
}
