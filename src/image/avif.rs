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
    
    let result = optimize_avif_with_quality(data, quality);
    
    let elapsed = get_current_time_ms() - start_time;
    update_performance_stats(true, elapsed, data_size);
    
    // Performance violation tracking
    let target = if data_size < 51200 { 100.0 } else { 150.0 }; // 50KB threshold
    if elapsed > target {
        unsafe {
            PERF_STATS.performance_target_violations += 1;
        }
        web_sys::console::warn_1(&format!("PERFORMANCE VIOLATION: AVIF processing took {:.1}ms (target: {:.1}ms) for {:.1}MB file", 
                                        elapsed, target, data_size as f64 / 1_000_000.0).into());
    }
    
    result.map_err(|e| wasm_bindgen::JsValue::from_str(&format!("{}", e)))
}

/// Optimize AVIF with quality-based approach
pub fn optimize_avif_with_quality(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    // First validate that this is actually AVIF
    if !is_avif(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid AVIF file".to_string()));
    }
    
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    config.lossless = quality > 95;
    
    optimize_avif(data, quality, &config)
}

/// Main AVIF optimization entry point
pub fn optimize_avif(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // For now, implement basic AVIF handling
    // In a full implementation, you'd use avif-parse, avif-serialize, or cavif
    
    if quality >= 80 {
        optimize_avif_high_quality(data, config)
    } else if quality >= 50 {
        optimize_avif_medium_quality(data, config)
    } else {
        optimize_avif_low_quality(data, config)
    }
}

fn optimize_avif_high_quality(data: &[u8], config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // For high quality, minimal optimization
    if !config.preserve_metadata {
        // Strip unnecessary metadata
        strip_avif_metadata(data)
    } else {
        Ok(data.to_vec())
    }
}

fn optimize_avif_medium_quality(data: &[u8], config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Medium quality: basic optimization
    let optimized = strip_avif_metadata(data)?;
    
    // Further optimization could be applied here
    Ok(optimized)
}

fn optimize_avif_low_quality(data: &[u8], config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Aggressive optimization
    let _ = config;
    
    // For low quality, might convert to more efficient format
    // But for now, just strip metadata
    strip_avif_metadata(data)
}

fn strip_avif_metadata(data: &[u8]) -> PixieResult<Vec<u8>> {
    // Basic AVIF metadata stripping
    // Real implementation would parse AVIF boxes and remove metadata boxes
    
    if data.len() < 12 {
        return Ok(data.to_vec());
    }
    
    // For now, return data as-is (placeholder)
    // Real AVIF optimization would require libavif or similar
    Ok(data.to_vec())
}

/// Check if data is AVIF format
pub fn is_avif(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    
    // AVIF files start with ftyp box containing 'avif'
    // Check for ftyp box (first 4 bytes are size, next 4 are 'ftyp')
    if &data[4..8] == b"ftyp" {
        // Look for 'avif' brand in the next bytes
        if data.len() >= 16 && &data[8..12] == b"avif" {
            return true;
        }
        
        // Also check for other AVIF-related brands
        for i in 8..data.len().saturating_sub(4) {
            if &data[i..i+4] == b"avif" || &data[i..i+4] == b"avis" {
                return true;
            }
        }
    }
    
    false
}

/// Convert any image format to AVIF (placeholder)
pub fn convert_any_format_to_avif(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    // This would require a full AVIF encoder like cavif
    // For now, return an error
    let _ = (data, quality);
    Err(PixieError::FeatureNotAvailable("AVIF encoding not yet implemented".to_string()))
}

/// Convert AVIF to compressed format (fallback to other formats)
pub fn convert_avif_to_compressed(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        // Try to load AVIF and convert to WebP or PNG
        match load_from_memory(data) {
            Ok(img) => {
                let mut output = Vec::new();
                
                // Convert to PNG for compatibility
                use image::codecs::png::PngEncoder;
                use image::ImageEncoder;
                
                let encoder = PngEncoder::new(&mut output);
                encoder.write_image(&img.to_rgba8(), img.width(), img.height(), image::ExtendedColorType::Rgba8)
                    .map_err(|e| PixieError::ProcessingError(format!("PNG conversion failed: {}", e)))?;
                
                Ok(output)
            },
            Err(_) => {
                // Fallback: return original
                Ok(data.to_vec())
            }
        }
    }
    
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, quality);
        Ok(data.to_vec())
    }
}

/// Legacy function for backward compatibility
pub fn optimize_avif_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_avif_with_quality(data, quality)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avif_detection() {
        // Valid AVIF header
        let avif_data = [
            0x00, 0x00, 0x00, 0x20, // Box size
            b'f', b't', b'y', b'p',  // ftyp
            b'a', b'v', b'i', b'f',  // avif brand
            0x00, 0x00, 0x00, 0x00,  // Minor version
        ];
        assert!(is_avif(&avif_data));
        
        // Invalid data
        let invalid_data = [0x00, 0x01, 0x02, 0x03];
        assert!(!is_avif(&invalid_data));
        
        // PNG data (should not be detected as AVIF)
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(!is_avif(&png_data));
    }

    #[test]
    fn test_avif_optimization() {
        let avif_data = [
            0x00, 0x00, 0x00, 0x20,
            b'f', b't', b'y', b'p',
            b'a', b'v', b'i', b'f',
            0x00, 0x00, 0x00, 0x00,
            // Additional dummy data
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
        ];
        
        let result = optimize_avif_with_quality(&avif_data, 75);
        assert!(result.is_ok());
    }
}
