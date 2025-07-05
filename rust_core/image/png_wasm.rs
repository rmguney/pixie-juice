//! WASM-compatible PNG optimization using the image crate

use crate::types::{OptConfig, OptError, OptResult};
use image::{ImageFormat};
use std::io::Cursor;

/// Optimize PNG files using the image crate (WASM compatible)
pub fn optimize_png(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load the image
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to load PNG: {}", e)))?;
    
    // Use the image crate's built-in PNG encoder
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    
    // Re-encode with PNG format
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| OptError::ProcessingError(format!("Failed to encode PNG: {}", e)))?;
    
    // Basic optimization: if lossless, just return the re-encoded image
    // For WASM builds, we don't have access to advanced PNG optimizers like oxipng
    if config.lossless.unwrap_or(false) {
        Ok(output)
    } else {
        // For non-lossless optimization, we can reduce quality by converting to RGB and back
        let rgb_img = img.into_rgb8();
        let mut optimized_output = Vec::new();
        let mut opt_cursor = Cursor::new(&mut optimized_output);
        
        image::DynamicImage::ImageRgb8(rgb_img)
            .write_to(&mut opt_cursor, ImageFormat::Png)
            .map_err(|e| OptError::ProcessingError(format!("Failed to optimize PNG: {}", e)))?;
        
        Ok(optimized_output)
    }
}

/// Basic PNG validation
pub fn validate_png(data: &[u8]) -> OptResult<()> {
    image::load_from_memory_with_format(data, ImageFormat::Png)
        .map_err(|e| OptError::ProcessingError(format!("Invalid PNG: {}", e)))?;
    Ok(())
}
