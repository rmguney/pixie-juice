//! Mesh format handling

extern crate alloc;
use alloc::{format, string::ToString};
use crate::{OptError, OptResult};
// WASM-compatible format detection - no std::path usage

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    // Phase 2: Core Mesh Formats (implemented)
    Obj,
    Ply,
    Gltf,
    Glb,
    // Phase 2: Planned formats (not yet implemented)
    // Stl,
    // Dae,
    // Fbx,
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
            // Phase 2: Planned formats (not yet implemented)
            // "stl" => Ok(Self::Stl),
            // "dae" => Ok(Self::Dae),
            // "fbx" => Ok(Self::Fbx),
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
            // Phase 2: Planned formats (not yet implemented)
            // Self::Stl => "stl",
            // Self::Dae => "dae",
            // Self::Fbx => "fbx",
        }
    }
    
    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Obj => "model/obj",
            Self::Ply => "model/ply",
            Self::Gltf => "model/gltf+json",
            Self::Glb => "model/gltf-binary",
            // Phase 2: Planned formats (not yet implemented)
            // Self::Stl => "model/stl",
            // Self::Dae => "model/vnd.collada+xml",
            // Self::Fbx => "model/fbx",
        }
    }
    
    /// Check if format supports binary encoding
    pub fn supports_binary(&self) -> bool {
        matches!(self, Self::Ply | Self::Glb)  // Removed Self::Stl (not implemented)
    }
    
    /// Check if format supports materials/textures
    pub fn supports_materials(&self) -> bool {
        matches!(self, Self::Obj | Self::Gltf | Self::Glb)  // Removed Self::Dae | Self::Fbx (not implemented)
    }
    
    /// Check if format is suitable for web use
    pub fn web_compatible(&self) -> bool {
        matches!(self, Self::Gltf | Self::Glb)
    }
}
