//! Legacy optimization wrappers - being phased out in favor of dedicated modules
//! 
//! This module provides compatibility wrappers for mesh and video optimization.
//! Image optimization has been moved to the image module with Rust-first implementation.

use crate::types::{OptConfig, OptResult};

/// Mesh optimizer using Rust-first approach with C hotspots for decimation
pub struct MeshOptimizer;

impl MeshOptimizer {
    pub fn new() -> Self {
        Self
    }
    
    /// Optimize mesh data based on configuration
    pub fn optimize(&self, input_data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
        // This is a legacy compatibility layer - real mesh optimization happens in mesh::optimizer
        // For now, just pass through the data since this API is being deprecated
        log::info!("Legacy mesh optimization called - consider using mesh::optimizer directly");
        let _ = config; // Suppress unused warning
        Ok(input_data.to_vec())
    }
    
    /// Deduplicate vertices in mesh using Rust implementation
    pub fn deduplicate_vertices(&self, input_data: &[u8]) -> OptResult<Vec<u8>> {
        // Vertex deduplication is now implemented in mesh::optimizer
        log::info!("Legacy vertex deduplication called - consider using mesh::optimizer directly");
        Ok(input_data.to_vec())
    }
    
    /// Reduce triangle count using C hotspot for performance
    pub fn reduce_triangles(&self, input_data: &[u8], ratio: f32) -> OptResult<Vec<u8>> {
        // Triangle reduction is now implemented in mesh::optimizer using C hotspots
        log::info!("Legacy triangle reduction called (ratio: {}) - consider using mesh::optimizer directly", ratio);
        Ok(input_data.to_vec())
    }
    
    /// Validate mesh data using Rust
    pub fn validate_mesh(&self, _input_data: &[u8]) -> OptResult<bool> {
        // Mesh validation is now implemented in mesh::validator and mesh::optimizer
        log::info!("Legacy mesh validation called - consider using mesh::validator directly");
        Ok(true)
    }
}

impl Default for MeshOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Video optimizer using Rust-first approach with C hotspots for encoding
pub struct VideoOptimizer;

impl VideoOptimizer {
    pub fn new() -> Self {
        Self
    }
    
    /// Optimize video data based on configuration  
    pub fn optimize(&self, input_data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
        // Video optimization using Rust-first approach with C hotspots for encoding
        // For now, this is a placeholder since video optimization is complex and requires FFmpeg integration
        log::info!("Video optimization called - this is a placeholder implementation");
        let _ = config; // Suppress unused warning
        Ok(input_data.to_vec())
    }
    
    /// Validate video format using Rust (would need ffmpeg-next or similar)
    pub fn validate_format(&self, input_data: &[u8]) -> OptResult<String> {
        // Basic format detection based on file signatures
        if input_data.len() < 8 {
            return Ok("unknown".to_string());
        }
        
        // Check common video format signatures
        if input_data.starts_with(b"\x00\x00\x00\x18ftypmp4") ||
           input_data.starts_with(b"\x00\x00\x00\x20ftypiso") {
            Ok("mp4".to_string())
        } else if input_data.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
            Ok("webm".to_string())
        } else if input_data.starts_with(b"RIFF") && input_data.len() >= 12 && &input_data[8..12] == b"AVI " {
            Ok("avi".to_string())
        } else {
            Ok("unknown".to_string())
        }
    }
    
    /// Compress video with CRF using C hotspot (would need FFmpeg integration)
    pub fn compress_crf(&self, input_data: &[u8], crf: i32) -> OptResult<Vec<u8>> {
        // This would require FFmpeg integration through C hotspots
        // For now, return input data as placeholder
        log::info!("Video compression called with CRF: {} - this is a placeholder implementation", crf);
        Ok(input_data.to_vec())
    }
}

impl Default for VideoOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
