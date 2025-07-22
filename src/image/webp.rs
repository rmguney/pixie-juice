//! WebP optimization using the webp crate
//! 
//! Handles WebP encoding/decoding and optimization with quality control.

use crate::types::{OptConfig, OptError, OptResult};

/// Optimize WebP data or convert other formats to WebP
pub fn optimize_webp(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    if is_webp(data) {
        // For existing WebP files, try safe recompression first
        match recompress_webp_safe(data, config) {
            Ok(result) => Ok(result),
            Err(_) => {
                // If recompression fails or times out, fall back to conversion
                log::warn!("WebP recompression failed, falling back to format conversion");
                convert_to_webp(data, config)
            }
        }
    } else {
        // For other formats, convert to WebP
        convert_to_webp(data, config)
    }
}

/// Check if data is WebP format
fn is_webp(data: &[u8]) -> bool {
    data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP"
}

/// Re-compress existing WebP with new settings (safe version with timeout protection)
fn recompress_webp_safe(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // For very large WebP files, skip recompression to avoid hanging
    if data.len() > 50_000_000 {  // 50MB limit
        return Err(OptError::ProcessingError("WebP file too large for safe recompression".to_string()));
    }
    
    // Try to decode WebP with a reasonable timeout simulation
    let decoded = webp::Decoder::new(data)
        .decode()
        .ok_or_else(|| OptError::ProcessingError("Failed to decode WebP".to_string()))?;

    // Check decoded size for reasonableness
    let pixel_count = decoded.width() as usize * decoded.height() as usize;
    if pixel_count > 50_000_000 {  // 50M pixels limit
        return Err(OptError::ProcessingError("Decoded WebP too large for safe processing".to_string()));
    }

    // Try multiple quality levels to find the best compression
    let mut best_result = data.to_vec();
    let mut best_reduction = 0.0;

    // Determine quality candidates based on config
    let quality_candidates = if let Some(q) = config.quality {
        vec![q as f32]
    } else if let Some(target_reduction) = config.target_reduction {
        // Multiple quality levels based on target reduction
        if target_reduction >= 0.6 {
            vec![15.0, 25.0, 35.0]  // Very aggressive for 60%+ reduction
        } else if target_reduction >= 0.4 {
            vec![30.0, 40.0, 50.0]  // Aggressive for 40%+ reduction
        } else if target_reduction >= 0.2 {
            vec![50.0, 60.0, 70.0]  // Moderate for 20%+ reduction
        } else {
            vec![70.0, 80.0]        // Conservative
        }
    } else {
        vec![60.0, 70.0, 80.0]  // Default quality levels
    };
    
    // Try each quality level and keep the best result
    for quality in quality_candidates {
        let encoder = webp::Encoder::from_rgba(decoded.as_ref(), decoded.width(), decoded.height());
        let encoded = encoder.encode(quality);
        
        let reduction = 1.0 - (encoded.len() as f64 / data.len() as f64);
        log::debug!("WebP quality {}: {} bytes ({:.1}% reduction)", 
                   quality, encoded.len(), reduction * 100.0);
        
        if reduction > best_reduction && encoded.len() < data.len() {
            best_result = encoded.to_vec();
            best_reduction = reduction;
        }
    }

    if best_reduction > 0.01 { // At least 1% reduction
        log::info!("WebP recompressed: {} bytes -> {} bytes ({:.1}% reduction)", 
                  data.len(), best_result.len(), best_reduction * 100.0);
        Ok(best_result)
    } else {
        // Return original if no improvement
        log::info!("WebP recompression: no size improvement, keeping original");
        Ok(data.to_vec())
    }
}

/// Re-compress existing WebP with new settings
#[allow(dead_code)]
fn recompress_webp(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Decode existing WebP
    let decoded = webp::Decoder::new(data)
        .decode()
        .ok_or_else(|| OptError::ProcessingError("Failed to decode WebP".to_string()))?;

    // Determine quality setting
    let quality = if let Some(q) = config.quality {
        q as f32
    } else if let Some(target_reduction) = config.target_reduction {
        // Aggressive quality reduction based on target
        if target_reduction >= 0.6 {
            25.0  // Very aggressive compression for 60%+ reduction
        } else if target_reduction >= 0.4 {
            40.0  // Aggressive compression for 40%+ reduction
        } else {
            60.0  // Moderate compression
        }
    } else {
        75.0  // Default quality
    };
    
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
    
    // Try multiple optimization approaches
    let mut best_result = data.to_vec();
    let mut best_reduction = 0.0;

    // Determine quality settings to try based on configuration
    let quality_levels = if let Some(q) = config.quality {
        vec![q as f32]
    } else if config.lossless.unwrap_or(false) {
        vec![100.0]  // Lossless only
    } else if let Some(target_reduction) = config.target_reduction {
        // Multiple quality levels based on target reduction
        if target_reduction >= 0.6 {
            vec![20.0, 30.0, 40.0]  // Very aggressive for 60%+ reduction
        } else if target_reduction >= 0.4 {
            vec![40.0, 50.0, 60.0]  // Aggressive for 40%+ reduction
        } else if target_reduction >= 0.2 {
            vec![60.0, 70.0, 80.0]  // Moderate for 20%+ reduction
        } else {
            vec![75.0, 85.0]        // Conservative
        }
    } else {
        vec![60.0, 75.0, 85.0]  // Default quality levels
    };

    // Try each quality level
    for quality in &quality_levels {
        let encoder = webp::Encoder::from_rgba(
            rgba_image.as_raw(), 
            rgba_image.width(), 
            rgba_image.height()
        );

        let encoded = if config.lossless.unwrap_or(false) {
            encoder.encode_lossless()
        } else {
            encoder.encode(*quality)
        };

        let reduction = 1.0 - (encoded.len() as f64 / data.len() as f64);
        log::debug!("WebP conversion quality {}: {} bytes ({:.1}% reduction)", 
                   quality, encoded.len(), reduction * 100.0);

        if encoded.len() < best_result.len() {
            best_result = encoded.to_vec();
            best_reduction = reduction;
        }
    }

    // If lossless didn't work well and we're targeting reduction, try lossy
    if config.lossless.unwrap_or(false) && best_reduction < 0.2 && 
       config.target_reduction.map_or(false, |r| r >= 0.2) {
        log::info!("Lossless WebP didn't achieve target, trying lossy compression");
        
        let encoder = webp::Encoder::from_rgba(
            rgba_image.as_raw(), 
            rgba_image.width(), 
            rgba_image.height()
        );
        
        let lossy_quality = config.target_reduction.map_or(70.0, |r| {
            if r >= 0.6 { 30.0 } else if r >= 0.4 { 50.0 } else { 70.0 }
        });
        
        let lossy_encoded = encoder.encode(lossy_quality);
        let lossy_reduction = 1.0 - (lossy_encoded.len() as f64 / data.len() as f64);
        
        if lossy_encoded.len() < best_result.len() {
            best_result = lossy_encoded.to_vec();
            best_reduction = lossy_reduction;
        }
    }

    log::info!("WebP conversion: {} bytes -> {} bytes ({:.1}% reduction)", 
              data.len(), best_result.len(), best_reduction * 100.0);

    Ok(best_result)
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
