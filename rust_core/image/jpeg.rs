//! JPEG optimization using jpeg-encoder and optional mozjpeg
//! 
//! Provides quality-based JPEG optimization with optional mozjpeg integration
//! for better compression ratios.

use crate::types::{OptConfig, OptError, OptResult};

/// Optimize JPEG data
pub fn optimize_jpeg(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // First, decode the JPEG to get raw image data
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to decode JPEG: {}", e)))?;

    let rgb_image = img.to_rgb8();
    let (_width, _height) = (rgb_image.width(), rgb_image.height());

    // Determine quality setting
    let quality = config.quality.unwrap_or(85).clamp(1, 100) as u8;

    // Use mozjpeg if available (native builds only)
    #[cfg(feature = "native")]
    if let Ok(optimized) = optimize_with_mozjpeg(&rgb_image, quality) {
        log::info!("JPEG optimized with mozjpeg: {} bytes -> {} bytes ({:.1}% reduction)", 
                  data.len(), 
                  optimized.len(),
                  (1.0 - (optimized.len() as f64 / data.len() as f64)) * 100.0);
        return Ok(optimized);
    }

    // Fallback to pure Rust JPEG encoder
    optimize_with_rust_encoder(&rgb_image, quality, data.len())
}

/// Optimize JPEG using pure Rust encoder
fn optimize_with_rust_encoder(
    rgb_image: &image::RgbImage, 
    quality: u8,
    original_size: usize
) -> OptResult<Vec<u8>> {
    let mut output = Vec::new();
    
    let encoder = jpeg_encoder::Encoder::new(&mut output, quality);
    encoder.encode(
        rgb_image.as_raw(),
        rgb_image.width() as u16,
        rgb_image.height() as u16,
        jpeg_encoder::ColorType::Rgb
    ).map_err(|e| OptError::ProcessingError(format!("JPEG encoding failed: {}", e)))?;

    log::info!("JPEG optimized with Rust encoder: {} bytes -> {} bytes ({:.1}% reduction)", 
              original_size, 
              output.len(),
              (1.0 - (output.len() as f64 / original_size as f64)) * 100.0);

    Ok(output)
}

/// Optimize JPEG using mozjpeg (native only, better compression)
#[cfg(feature = "native")]
fn optimize_with_mozjpeg(rgb_image: &image::RgbImage, quality: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // This would integrate with mozjpeg-sys for better compression
    // For now, return error to fall back to Rust encoder
    Err("mozjpeg integration not implemented yet".into())
}

/// Check if JPEG can be optimized
pub fn can_optimize_jpeg(data: &[u8]) -> bool {
    data.len() >= 3 && data.starts_with(&[0xFF, 0xD8, 0xFF])
}

/// Extract JPEG quality estimate from existing file
pub fn estimate_jpeg_quality(data: &[u8]) -> OptResult<u8> {
    if !can_optimize_jpeg(data) {
        return Err(OptError::InvalidFormat("Not a valid JPEG file".to_string()));
    }

    // This is a simplified quality estimation
    // In practice, you'd parse the quantization tables to estimate quality
    // For now, return a conservative estimate
    Ok(85)
}

/// Get JPEG-specific information
pub fn get_jpeg_info(data: &[u8]) -> OptResult<JpegInfo> {
    if !can_optimize_jpeg(data) {
        return Err(OptError::InvalidFormat("Not a valid JPEG file".to_string()));
    }

    // Parse JPEG markers to extract basic information
    let mut pos = 2; // Skip SOI marker (FF D8)
    let mut width = 0u16;
    let mut height = 0u16;
    
    while pos + 4 < data.len() {
        if data[pos] != 0xFF {
            break;
        }
        
        let marker = data[pos + 1];
        let length = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
        
        // SOF markers contain image dimensions
        if matches!(marker, 0xC0..=0xCF) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
            if pos + 9 < data.len() {
                height = u16::from_be_bytes([data[pos + 5], data[pos + 6]]);
                width = u16::from_be_bytes([data[pos + 7], data[pos + 8]]);
                break;
            }
        }
        
        pos += 2 + length as usize;
    }

    if width == 0 || height == 0 {
        return Err(OptError::ProcessingError("Could not parse JPEG dimensions".to_string()));
    }

    Ok(JpegInfo {
        width: width as u32,
        height: height as u32,
        estimated_quality: estimate_jpeg_quality(data).unwrap_or(85),
        file_size: data.len(),
    })
}

#[derive(Debug, Clone)]
pub struct JpegInfo {
    pub width: u32,
    pub height: u32,
    pub estimated_quality: u8,
    pub file_size: usize,
}
