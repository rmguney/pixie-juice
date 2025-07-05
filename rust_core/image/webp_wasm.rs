//! WASM-compatible WebP processing using the image crate

use crate::types::{OptConfig, OptError, OptResult};
use image::{ImageFormat, DynamicImage};
use std::io::Cursor;

/// Convert to WebP format using the image crate's built-in support
/// Note: WASM builds have limited WebP support compared to native builds
pub fn optimize_webp(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load the image
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to load image for WebP conversion: {}", e)))?;
    
    // Check if the image crate supports WebP encoding
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    
    // Try to encode as WebP, fallback to PNG if not supported
    match img.write_to(&mut cursor, ImageFormat::WebP) {
        Ok(_) => Ok(output),
        Err(_) => {
            // WebP encoding not available, convert to PNG instead
            output.clear();
            cursor = Cursor::new(&mut output);
            img.write_to(&mut cursor, ImageFormat::Png)
                .map_err(|e| OptError::ProcessingError(format!("Failed to encode as PNG fallback: {}", e)))?;
            Ok(output)
        }
    }
}

/// Convert from WebP format using the image crate
pub fn convert_from_webp(data: &[u8], _config: &OptConfig) -> OptResult<DynamicImage> {
    image::load_from_memory_with_format(data, ImageFormat::WebP)
        .map_err(|e| OptError::ProcessingError(format!("Failed to load WebP: {}", e)))
}

/// Convert various formats to WebP
pub fn convert_to_webp(data: &[u8], _config: &OptConfig) -> OptResult<Vec<u8>> {
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to load image: {}", e)))?;
    
    let mut output = Vec::new();
    let mut cursor = Cursor::new(&mut output);
    
    // Try WebP encoding, fallback to PNG
    match img.write_to(&mut cursor, ImageFormat::WebP) {
        Ok(_) => Ok(output),
        Err(_) => {
            // WebP not supported, use PNG instead
            output.clear();
            cursor = Cursor::new(&mut output);
            img.write_to(&mut cursor, ImageFormat::Png)
                .map_err(|e| OptError::ProcessingError(format!("Failed to encode as PNG: {}", e)))?;
            Ok(output)
        }
    }
}

/// Basic WebP validation
pub fn validate_webp(data: &[u8]) -> OptResult<()> {
    image::load_from_memory_with_format(data, ImageFormat::WebP)
        .map_err(|e| OptError::ProcessingError(format!("Invalid WebP: {}", e)))?;
    Ok(())
}
