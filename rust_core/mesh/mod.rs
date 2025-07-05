//! Mesh processing module - Rust-first with C performance hotspots
//! 
//! This module handles 3D mesh optimization using Rust for I/O and validation,
//! with C hotspots only for performance-critical mesh decimation algorithms.

pub mod formats;
pub mod gltf;
pub mod fbx;
pub mod dae;
pub mod obj;
pub mod stl;
pub mod ply;
pub mod usdz;
pub mod loader;
pub mod validator;
pub mod optimizer;

pub use formats::*;
use crate::types::{OptConfig, OptResult};

/// Universal mesh optimizer that dispatches to format-specific processors
pub struct MeshOptimizer;

impl MeshOptimizer {
    pub fn new() -> Self {
        Self
    }

    /// Optimize a mesh based on its detected format
    pub fn optimize(&self, data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
        let format = detect_mesh_format(data)?;
        
        match format {
            MeshFormat::GLTF => gltf::optimize_gltf(data, config),
            MeshFormat::GLB => gltf::optimize_glb(data, config),
            MeshFormat::OBJ => obj::optimize_obj(data, config),
            MeshFormat::STL => stl::optimize_stl(data, config),
            MeshFormat::PLY => ply::optimize_ply(data, config),
            MeshFormat::FBX => fbx::optimize_fbx(data, config),
            MeshFormat::DAE => dae::optimize_dae(data, config),
            MeshFormat::USDZ => usdz::optimize_usdz(data, config),
        }
    }

    /// Get info about a mesh without loading it fully
    pub fn get_info(&self, data: &[u8]) -> OptResult<MeshInfo> {
        formats::get_mesh_info(data)
    }
    
    /// Validate a mesh file and return a detailed report
    pub fn validate(&self, data: &[u8]) -> OptResult<validator::ValidationReport> {
        // First detect the format
        let format = detect_mesh_format(data)?;
        
        // Load the mesh data using the general loader
        let mesh_data = loader::load_mesh_data(data, format.extension())?;
        
        // Run validation on the loaded data
        let validator = validator::MeshValidator::new();
        validator.validate(&mesh_data)
    }
    
    /// Quick format detection and basic validation
    pub fn detect_and_validate_format(&self, data: &[u8]) -> OptResult<(MeshFormat, bool)> {
        let format = detect_mesh_format(data)?;
        
        // Quick validation - just try to parse without full validation
        let is_valid = match self.validate(data) {
            Ok(report) => report.is_valid,
            Err(_) => false,
        };
        
        Ok((format, is_valid))
    }
}

impl Default for MeshOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MeshInfo {
    pub format: MeshFormat,
    pub vertex_count: u32,
    pub triangle_count: u32,
    pub file_size: usize,
}
