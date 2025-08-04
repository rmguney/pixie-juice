//! TGA (Targa) format support using WASM-compatible Rust libraries

extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{PixieResult, ImageOptConfig, PixieError};
use crate::optimizers::{get_current_time_ms, update_performance_stats, PERF_STATS};

#[cfg(feature = "image")]
use image::load_from_memory;

/// Entry point for TGA optimization with performance tracking
pub fn optimize_tga_entry(data: &[u8], quality: u8) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let start_time = get_current_time_ms();
    let data_size = data.len();
    
    let result = optimize_tga_with_quality(data, quality);
    
    let elapsed = get_current_time_ms() - start_time;
    update_performance_stats(true, elapsed, data_size);
    
    // Performance violation tracking
    let target = if data_size < 51200 { 100.0 } else { 150.0 }; // 50KB threshold
    if elapsed > target {
        unsafe {
            PERF_STATS.performance_target_violations += 1;
        }
        web_sys::console::warn_1(&format!("PERFORMANCE VIOLATION: TGA processing took {:.1}ms (target: {:.1}ms) for {:.1}MB file", 
                                        elapsed, target, data_size as f64 / 1_000_000.0).into());
    }
    
    result.map_err(|e| wasm_bindgen::JsValue::from_str(&format!("{}", e)))
}

/// Optimize TGA with quality-based approach
pub fn optimize_tga_with_quality(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    // First validate that this is actually TGA
    if !is_tga(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid TGA file".to_string()));
    }
    
    let mut config = ImageOptConfig::default();
    config.quality = quality;
    config.lossless = quality > 95;
    
    optimize_tga(data, quality, &config)
}

/// Main TGA optimization entry point
pub fn optimize_tga(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // TGA optimization strategies based on quality
    if quality >= 80 {
        optimize_tga_high_quality(data, config)
    } else if quality >= 50 {
        optimize_tga_medium_quality(data, config)
    } else {
        optimize_tga_low_quality(data, config)
    }
}

fn optimize_tga_high_quality(data: &[u8], config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // For high quality, minimal optimization
    if !config.preserve_metadata {
        // Strip unnecessary metadata and comments
        strip_tga_metadata(data)
    } else {
        Ok(data.to_vec())
    }
}

fn optimize_tga_medium_quality(data: &[u8], config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Medium quality: convert to more efficient format if beneficial
    if data.len() > 10240 { // > 10KB
        // Convert to PNG for better compression
        convert_tga_to_png(data, config)
    } else {
        // Keep as TGA but optimize
        strip_tga_metadata(data)
    }
}

fn optimize_tga_low_quality(data: &[u8], config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Aggressive optimization: always convert to more efficient format
    convert_tga_to_png(data, config)
}

fn convert_tga_to_png(data: &[u8], _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load TGA for PNG conversion: {}", e)))?;
        
        // Convert to PNG with appropriate settings
        let mut png_data = Vec::new();
        
        // Use image crate's built-in encoding without std::io::Cursor
        use image::codecs::png::PngEncoder;
        use image::ImageEncoder;
        
        let encoder = PngEncoder::new(&mut png_data);
        encoder.write_image(&img.to_rgba8(), img.width(), img.height(), image::ExtendedColorType::Rgba8)
            .map_err(|e| PixieError::ProcessingError(format!("PNG encoding failed: {}", e)))?;
        
        Ok(png_data)
    }
    
    #[cfg(not(feature = "image"))]
    {
        // Fallback: just strip metadata
        strip_tga_metadata(data)
    }
}

fn strip_tga_metadata(data: &[u8]) -> PixieResult<Vec<u8>> {
    if data.len() < 18 {
        return Ok(data.to_vec());
    }
    
    // TGA header is 18 bytes, followed by optional ID field
    let id_length = data[0] as usize;
    let header_size = 18 + id_length;
    
    if data.len() < header_size {
        return Ok(data.to_vec());
    }
    
    // Create new TGA without ID field (metadata)
    let mut optimized = Vec::with_capacity(data.len());
    
    // Copy header but set ID length to 0
    optimized.push(0); // ID length = 0
    optimized.extend_from_slice(&data[1..18]); // Rest of header
    
    // Skip original ID field and copy image data
    optimized.extend_from_slice(&data[header_size..]);
    
    Ok(optimized)
}

/// Check if data is TGA format
pub fn is_tga(data: &[u8]) -> bool {
    if data.len() < 18 {
        return false;
    }
    
    // TGA has no fixed magic bytes, but we can check header structure
    let id_length = data[0];
    let color_map_type = data[1];
    let image_type = data[2];
    
    // Color map type should be 0 or 1
    if color_map_type > 1 {
        return false;
    }
    
    // Image type should be valid (0-11, with gaps)
    match image_type {
        0 | 1 | 2 | 3 | 9 | 10 | 11 => {},
        _ => return false,
    }
    
    // Check if header fields are reasonable
    let width = u16::from_le_bytes([data[12], data[13]]);
    let height = u16::from_le_bytes([data[14], data[15]]);
    let bpp = data[16];
    
    // Reasonable dimensions and bit depth
    width > 0 && height > 0 && 
    width <= 65535 && height <= 65535 &&
    (bpp == 8 || bpp == 15 || bpp == 16 || bpp == 24 || bpp == 32) &&
    data.len() >= (18 + id_length as usize)
}

/// Convert any image format to TGA
pub fn convert_any_format_to_tga(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load image for TGA conversion: {}", e)))?;
        
        // Convert to TGA format using image encoder
        let mut tga_data = Vec::new();
        
        use image::codecs::tga::TgaEncoder;
        use image::ImageEncoder;
        
        let encoder = TgaEncoder::new(&mut tga_data);
        
        // Apply quality-based optimization
        if quality < 50 {
            // Low quality: reduce color depth
            let rgb_img = img.to_rgb8();
            encoder.write_image(&rgb_img, img.width(), img.height(), image::ExtendedColorType::Rgb8)
        } else {
            // Higher quality: use full color depth
            encoder.write_image(&img.to_rgba8(), img.width(), img.height(), image::ExtendedColorType::Rgba8)
        }.map_err(|e| PixieError::ProcessingError(format!("TGA encoding failed: {}", e)))?;
        
        Ok(tga_data)
    }
    
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, quality);
        Err(PixieError::FeatureNotAvailable("TGA encoding requires image feature".to_string()))
    }
}

/// Legacy function for backward compatibility
pub fn optimize_tga_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_tga_with_quality(data, quality)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tga_detection() {
        // Valid TGA header (uncompressed RGB)
        let tga_data = [
            0x00, // ID length
            0x00, // Color map type
            0x02, // Image type (uncompressed RGB)
            0x00, 0x00, 0x00, 0x00, 0x00, // Color map spec
            0x00, 0x00, 0x00, 0x00, // X/Y origin
            0x10, 0x00, // Width (16)
            0x10, 0x00, // Height (16)  
            0x18, // Bits per pixel (24)
            0x00, // Image descriptor
        ];
        assert!(is_tga(&tga_data));
        
        // Invalid data
        let invalid_data = [0x00, 0x01, 0x02, 0x03];
        assert!(!is_tga(&invalid_data));
        
        // PNG data (should not be detected as TGA)
        let png_data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(!is_tga(&png_data));
    }

    #[test]
    fn test_tga_optimization() {
        let tga_data = [
            0x05, // ID length = 5 (has metadata)
            0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x10, 0x00, 0x10, 0x00, 0x18, 0x00,
            // ID field (metadata)
            b'T', b'E', b'S', b'T', b'!',
            // Dummy image data
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
        ];
        
        // Should strip metadata for medium quality
        let result = optimize_tga_with_quality(&tga_data, 50);
        assert!(result.is_ok());
        let optimized = result.unwrap();
        
        // First byte should be 0 (no ID field)
        assert_eq!(optimized[0], 0);
    }
}
