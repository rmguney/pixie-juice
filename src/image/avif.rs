//! AVIF format support using WASM-compatible Rust libraries

extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{PixieResult, ImageOptConfig, PixieError};
use crate::optimizers::{get_current_time_ms, update_performance_stats, PERF_STATS};

#[cfg(feature = "image")]
use image::load_from_memory;

/// Entry point for AVIF optimization with performance tracking
pub fn optimize_avif_entry(data: &[u8], quality: u8) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let start_time = get_current_time_ms();
    let data_size = data.len();

    let result = optimize_avif(data, quality);
    
    match result {
        Ok(optimized_data) => {
            let elapsed_ms = get_current_time_ms() - start_time;
            update_performance_stats(true, elapsed_ms, optimized_data.len());
            Ok(optimized_data)
        }
        Err(e) => {
            unsafe {
                PERF_STATS.performance_target_violations += 1;
            }
            Err(wasm_bindgen::JsValue::from_str(&format!("AVIF optimization failed: {}", e)))
        }
    }
}

/// Main AVIF optimization function
pub fn optimize_avif(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let config = ImageOptConfig::with_quality(quality);

    // Try stripping metadata first for basic optimization
    if let Ok(stripped) = strip_avif_metadata(data) {
        if stripped.len() < data.len() {
            return Ok(stripped);
        }
    }

    // Fallback: convert to other formats for better compatibility
    if config.lossless {
        return convert_avif_to_png(data);
    } else {
        return convert_avif_to_webp(data);
    }
}

/// Strip AVIF metadata to reduce file size
fn strip_avif_metadata(data: &[u8]) -> PixieResult<Vec<u8>> {
    // For now, return a copy with basic validation
    if is_avif(data) {
        // Basic metadata stripping - remove EXIF, XMP blocks if present
        // This is a simplified approach - real implementation would parse AVIF structure
        Ok(data.to_vec())
    } else {
        Err(PixieError::ProcessingError("Not a valid AVIF file".to_string()))
    }
}

fn convert_avif_to_webp(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to decode AVIF: {}", e)))?;
        
        let mut buffer = Vec::new();
        let rgb_img = img.to_rgb8();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut buffer);
        rgb_img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to encode WebP: {}", e)))?;
        
        Ok(buffer)
    }
    
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::ProcessingError("Image crate not available".to_string()))
    }
}

fn convert_avif_to_png(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to decode AVIF: {}", e)))?;
        
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to encode PNG: {}", e)))?;
        
        Ok(buffer)
    }
    
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::ProcessingError("Image crate not available".to_string()))
    }
}

/// Detect AVIF format by checking for AVIF signature
pub fn is_avif(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }

    // Check for AVIF file signature
    // AVIF files start with "....ftypavif" or "....ftypavis"
    if data.len() >= 12 {
        // Check for ftyp box at offset 4
        if &data[4..8] == b"ftyp" {
            // Check for AVIF brand
            if &data[8..12] == b"avif" || &data[8..12] == b"avis" {
                return true;
            }
        }
    }

    // Alternative check: look for AVIF brand in first 32 bytes
    if data.len() >= 32 {
        for i in 0..24 {
            if data.len() > i + 4 && &data[i..i+4] == b"avif" {
                return true;
            }
        }
    }

    false
}
