//! Pure Rust image processing module
//! This module handles all image optimization using mature libs

pub mod formats;
pub mod gif;
pub mod jpeg;

// Conditional modules based on target platform
#[cfg(not(target_arch = "wasm32"))]
pub mod png;
#[cfg(not(target_arch = "wasm32"))]
pub mod webp;

#[cfg(target_arch = "wasm32")]
#[path = "png_wasm.rs"]
pub mod png;
#[cfg(target_arch = "wasm32")]
#[path = "webp_wasm.rs"]
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
                // BMP files are typically uncompressed and benefit from conversion to PNG
                let img = image::load_from_memory(data)
                    .map_err(|e| OptError::ProcessingError(e.to_string()))?;
                let mut png_data = Vec::new();
                img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
                    .map_err(|e| OptError::ProcessingError(e.to_string()))?;
                
                // Then optimize the PNG
                png::optimize_png(&png_data, config)
            },
            ImageFormat::TIFF => {
                // TIFF files can be large and uncompressed, convert to PNG for better compression
                let img = image::load_from_memory(data)
                    .map_err(|e| OptError::ProcessingError(e.to_string()))?;
                let mut png_data = Vec::new();
                img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
                    .map_err(|e| OptError::ProcessingError(e.to_string()))?;
                
                // Then optimize the PNG
                png::optimize_png(&png_data, config)
            },
        }
    }

    /// Optimize and convert an image to a specific output format
    pub fn optimize_to_format(&self, data: &[u8], target_format: ImageFormat, config: &OptConfig) -> OptResult<Vec<u8>> {
        let input_format = detect_image_format(data)?;
        
        // If target format matches input format, just optimize
        if input_format == target_format {
            return self.optimize(data, config);
        }
        
        // Otherwise, convert format first, then optimize
        let converted_data = self.convert_format(data, target_format)?;
        
        // Optimize in the target format
        match target_format {
            ImageFormat::PNG => png::optimize_png(&converted_data, config),
            ImageFormat::JPEG => jpeg::optimize_jpeg(&converted_data, config),
            ImageFormat::WebP => webp::optimize_webp(&converted_data, config),
            ImageFormat::GIF => gif::optimize_gif(&converted_data, config),
            ImageFormat::BMP => Ok(converted_data), // BMP doesn't need optimization
            ImageFormat::TIFF => Ok(converted_data), // TIFF doesn't need optimization
        }
    }
    
    /// Convert image to a different format
    fn convert_format(&self, data: &[u8], target_format: ImageFormat) -> OptResult<Vec<u8>> {
        use image::ImageFormat as ImageCrateFormat;
        
        let img = image::load_from_memory(data)
            .map_err(|e| OptError::ProcessingError(format!("Failed to load image: {}", e)))?;
        
        let mut output = Vec::new();
        let format = match target_format {
            ImageFormat::PNG => ImageCrateFormat::Png,
            ImageFormat::JPEG => ImageCrateFormat::Jpeg,
            ImageFormat::WebP => ImageCrateFormat::WebP,
            ImageFormat::GIF => ImageCrateFormat::Gif,
            ImageFormat::BMP => ImageCrateFormat::Bmp,
            ImageFormat::TIFF => ImageCrateFormat::Tiff,
        };
        
        img.write_to(&mut std::io::Cursor::new(&mut output), format)
            .map_err(|e| OptError::ProcessingError(format!("Failed to convert to {:?}: {}", target_format, e)))?;
        
        Ok(output)
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
