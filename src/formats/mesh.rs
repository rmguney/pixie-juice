//! Mesh format handling

extern crate alloc;
use alloc::{format, string::ToString};
use crate::{OptError, OptResult};
// WASM-compatible format detection - no std::path usage

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    // Core mesh formats
    Obj,        // Wavefront OBJ - via tobj crate
    Gltf,       // glTF 2.0 JSON - via gltf crate  
    Glb,        // glTF 2.0 Binary - via gltf crate
    Ply,        // Stanford PLY - via ply-rs crate
    Stl,        // STL (STereoLithography) - via stl_io crate
    Fbx,        // Autodesk FBX - custom binary parser via nom
}

impl MeshFormat {
    /// Detect format from file extension
    pub fn from_extension(filename: &str) -> OptResult<Self> {
        let ext = filename
            .split('.')
            .last()
            .ok_or_else(|| OptError::FormatError("No file extension".to_string()))?;
        
        match ext.to_lowercase().as_str() {
            "obj" => Ok(Self::Obj),
            "ply" => Ok(Self::Ply),
            "gltf" => Ok(Self::Gltf),
            "glb" => Ok(Self::Glb),
            "stl" => Ok(Self::Stl),
            "fbx" => Ok(Self::Fbx),
            _ => Err(OptError::FormatError(format!("Unsupported mesh format: {}", ext))),
        }
    }
    
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Obj => "obj",
            Self::Ply => "ply",
            Self::Gltf => "gltf",
            Self::Glb => "glb",
            Self::Stl => "stl",
            Self::Fbx => "fbx",
        }
    }
    
    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Obj => "model/obj",
            Self::Ply => "model/ply",
            Self::Gltf => "model/gltf+json",
            Self::Glb => "model/gltf-binary",
            Self::Stl => "model/stl",
            Self::Fbx => "model/fbx",
        }
    }
    
    /// Check if format supports binary encoding
    pub fn supports_binary(&self) -> bool {
        matches!(self, Self::Ply | Self::Glb | Self::Stl | Self::Fbx)
    }
    
    /// Check if format supports materials/textures
    pub fn supports_materials(&self) -> bool {
        matches!(self, Self::Obj | Self::Gltf | Self::Glb | Self::Fbx)
    }
    
    /// Check if format is suitable for web use
    pub fn web_compatible(&self) -> bool {
        matches!(self, Self::Gltf | Self::Glb)
    }
}
