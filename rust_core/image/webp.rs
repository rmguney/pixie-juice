//! WebP optimization using the webp crate
//! 
//! Handles WebP encoding/decoding and optimization with quality control.

use crate::types::{OptConfig, OptError, OptResult};

/// Optimize WebP data or convert other formats to WebP
pub fn optimize_webp(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // If input is already WebP, try to re-encode with better settings
    if is_webp(data) {
        return recompress_webp(data, config);
    }

    // Convert other formats to WebP
    convert_to_webp(data, config)
}

/// Check if data is WebP format
fn is_webp(data: &[u8]) -> bool {
    data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP"
}

/// Re-compress existing WebP with new settings
fn recompress_webp(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Decode existing WebP
    let decoded = webp::Decoder::new(data)
        .decode()
        .ok_or_else(|| OptError::ProcessingError("Failed to decode WebP".to_string()))?;

    let quality = config.quality.unwrap_or(85) as f32;
    
    // Re-encode with new settings
    let encoder = webp::Encoder::from_rgba(decoded.as_ref(), decoded.width(), decoded.height());
    let encoded = encoder.encode(quality);

    log::info!("WebP recompressed: {} bytes -> {} bytes ({:.1}% reduction)", 
              data.len(), 
              encoded.len(),
              (1.0 - (encoded.len() as f64 / data.len() as f64)) * 100.0);

    Ok(encoded.to_vec())
}

/// Convert other image formats to WebP
fn convert_to_webp(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load image using the image crate
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to decode image: {}", e)))?;

    let rgba_image = img.to_rgba8();
    let (width, height) = (rgba_image.width(), rgba_image.height());
    let quality = config.quality.unwrap_or(85) as f32;

    // Encode as WebP
    let encoder = webp::Encoder::from_rgba(rgba_image.as_raw(), width, height);
    let encoded = if config.lossless.unwrap_or(false) {
        encoder.encode_lossless()
    } else {
        encoder.encode(quality)
    };

    log::info!("Image converted to WebP: {} bytes -> {} bytes ({:.1}% reduction)", 
              data.len(), 
              encoded.len(),
              (1.0 - (encoded.len() as f64 / data.len() as f64)) * 100.0);

    Ok(encoded.to_vec())
}

/// Get WebP-specific information
pub fn get_webp_info(data: &[u8]) -> OptResult<WebpInfo> {
    if !is_webp(data) {
        return Err(OptError::InvalidFormat("Not a valid WebP file".to_string()));
    }
    
    let decoded = webp::Decoder::new(data)
        .decode()
        .ok_or_else(|| OptError::ProcessingError("Failed to decode WebP for info".to_string()))?;

    Ok(WebpInfo {
        width: decoded.width(),
        height: decoded.height(),
        has_alpha: true, // WebP decoder returns RGBA by default
        is_lossless: detect_webp_lossless(data),
        file_size: data.len(),
    })
}

/// Detect if WebP is lossless (simplified detection)
fn detect_webp_lossless(data: &[u8]) -> bool {
    // This is a simplified check - in practice you'd parse the WebP headers
    // Look for VP8L chunk (lossless) vs VP8 chunk (lossy)
    if data.len() < 20 {
        return false;
    }
    
    // Search for VP8L signature (lossless)
    for i in 12..data.len().saturating_sub(4) {
        if &data[i..i+4] == b"VP8L" {
            return true;
        }
    }
    
    false
}

#[derive(Debug, Clone)]
pub struct WebpInfo {
    pub width: u32,
    pub height: u32,
    pub has_alpha: bool,
    pub is_lossless: bool,
    pub file_size: usize,
}
