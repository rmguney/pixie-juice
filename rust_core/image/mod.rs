//! Pure Rust image processing module
//! 
//! This module handles all image optimization using mature Rust crates:
//! - PNG: oxipng for best-in-class optimization
//! - JPEG: jpeg-encoder and mozjpeg for quality/size balance
//! - WebP: webp crate for modern format support
//! - GIF: gif crate with color quantization
//! - Universal: image crate for format detection and basic operations

pub mod formats;
pub mod gif;
pub mod jpeg;
pub mod png;
pub mod webp;

pub use formats::*;
use crate::types::{OptConfig, OptError, OptResult};

/// Universal image optimizer that dispatches to format-specific optimizers
pub struct ImageOptimizer;

impl ImageOptimizer {
    pub fn new() -> Self {
        Self
    }

    /// Optimize an image based on its detected format
    pub fn optimize(&self, data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
        let format = detect_image_format(data)?;
        
        match format {
            ImageFormat::PNG => png::optimize_png(data, config),
            ImageFormat::JPEG => jpeg::optimize_jpeg(data, config),
            ImageFormat::WebP => webp::optimize_webp(data, config),
            ImageFormat::GIF => gif::optimize_gif(data, config),
            ImageFormat::BMP => {
                // Convert BMP to PNG for optimization
                let img = image::load_from_memory(data)
                    .map_err(|e| OptError::ProcessingError(e.to_string()))?;
                let mut png_data = Vec::new();
                img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
                    .map_err(|e| OptError::ProcessingError(e.to_string()))?;
                png::optimize_png(&png_data, config)
            },
            ImageFormat::TIFF => {
                // Convert TIFF to PNG for optimization
                let img = image::load_from_memory(data)
                    .map_err(|e| OptError::ProcessingError(e.to_string()))?;
                let mut png_data = Vec::new();
                img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
                    .map_err(|e| OptError::ProcessingError(e.to_string()))?;
                png::optimize_png(&png_data, config)
            },
        }
    }

    /// Get info about an image without loading it fully
    pub fn get_info(&self, data: &[u8]) -> OptResult<ImageInfo> {
        formats::get_image_info(data)
    }
}

impl Default for ImageOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub bit_depth: u8,
    pub file_size: usize,
}
