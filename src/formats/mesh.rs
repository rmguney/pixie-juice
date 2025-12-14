extern crate alloc;
use alloc::{format, string::ToString};
use crate::{OptError, OptResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshFormat {
    Obj,
    Gltf,
    Glb,
    Ply,
    Stl,
    Fbx,
}

impl MeshFormat {
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
    
    pub fn supports_binary(&self) -> bool {
        matches!(self, Self::Ply | Self::Glb | Self::Stl | Self::Fbx)
    }
    
    pub fn supports_materials(&self) -> bool {
        matches!(self, Self::Obj | Self::Gltf | Self::Glb | Self::Fbx)
    }
    
    pub fn web_compatible(&self) -> bool {
        matches!(self, Self::Gltf | Self::Glb)
    }
}
