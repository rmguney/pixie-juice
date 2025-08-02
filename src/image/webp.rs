//! WebP optimization using library-first approach with image crate
//! 
//! This module provides WebP optimization using the proven `image` crate
//! for WASM compatibility, following library-first implementation principles.

extern crate alloc;
use alloc::{vec::Vec, format, string::ToString};

use crate::types::{PixieResult, PixieError, ImageOptConfig, OptResult, OptError};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage, ImageEncoder};
use image::codecs::webp::WebPEncoder;

/// Optimize WebP image using library-first approach with the image crate
pub fn optimize_webp_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_webp_with_config(data, quality, &ImageOptConfig::default())
}

/// Optimize WebP with configuration - library-first implementation
pub fn optimize_webp_with_config(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        // Load the WebP using the proven image crate
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(
                format!("Failed to load WebP: {}", e)
            ))?;
        
        // Strategy selection based on quality parameter
        let strategies = get_webp_optimization_strategies(quality, &img);
        
        let mut best_result = data.to_vec();
        let mut best_size = data.len();
        
        // Try each optimization strategy and keep the best result
        for strategy in strategies {
            if let Ok(optimized) = apply_webp_strategy(&img, strategy, quality, config) {
                if optimized.len() < best_size {
                    best_result = optimized;
                    best_size = best_result.len();
                }
            }
        }
        
        // Only return optimized version if it's actually smaller
        if best_result.len() < data.len() {
            Ok(best_result)
        } else {
            // If no optimization helped, try metadata stripping as fallback
            strip_webp_metadata(data, quality)
        }
    }
    
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, quality, config);
        Err(PixieError::FeatureNotEnabled("WebP optimization requires 'image' feature".to_string()))
    }
}

#[cfg(feature = "image")]
#[derive(Debug, Clone)]
enum WebPOptimizationStrategy {
    /// Re-encode as WebP with different quality settings
    ReencodeWebP { webp_quality: u8 },
    /// Convert to JPEG (for non-transparent images)
    ConvertToJPEG { jpeg_quality: u8 },
    /// Convert to PNG for lossless preservation
    ConvertToPNG,
    /// Strip metadata while keeping image data
    MetadataStripping,
}

#[cfg(feature = "image")]
fn get_webp_optimization_strategies(quality: u8, img: &DynamicImage) -> Vec<WebPOptimizationStrategy> {
    let mut strategies = Vec::new();
    
    // Check if image has transparency
    let has_transparency = match img {
        DynamicImage::ImageRgba8(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < 255)
        },
        DynamicImage::ImageRgba16(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < u16::MAX)
        },
        _ => false,
    };
    
    // Strategy 1: Always try metadata stripping first (fastest)
    strategies.push(WebPOptimizationStrategy::MetadataStripping);
    
    // Strategy 2: Re-encode WebP with optimized quality
    let webp_quality = match quality {
        0..=30 => 40,   // Aggressive compression for low quality
        31..=60 => 65,  // Moderate compression
        61..=80 => 80,  // Conservative compression
        _ => 90,        // High quality preservation
    };
    strategies.push(WebPOptimizationStrategy::ReencodeWebP { webp_quality });
    
    // Strategy 3: For images without transparency, try JPEG conversion
    if !has_transparency && quality <= 85 {
        let jpeg_quality = match quality {
            0..=30 => 50,
            31..=60 => 70,
            61..=80 => 85,
            _ => 90,
        };
        strategies.push(WebPOptimizationStrategy::ConvertToJPEG { jpeg_quality });
    }
    
    // Strategy 4: For high quality requirements, try lossless PNG
    if quality >= 90 {
        strategies.push(WebPOptimizationStrategy::ConvertToPNG);
    }
    
    strategies
}

#[cfg(feature = "image")]
fn apply_webp_strategy(
    img: &DynamicImage, 
    strategy: WebPOptimizationStrategy, 
    quality: u8,
    _config: &ImageOptConfig
) -> PixieResult<Vec<u8>> {
    match strategy {
        WebPOptimizationStrategy::ReencodeWebP { webp_quality: _ } => {
            // Use image crate's WebPEncoder for lossless WebP re-encoding
            let mut best_output = Vec::new();
            let mut best_size = usize::MAX;
            
            // Strategy 1: Standard WebP lossless re-encoding
            let mut webp_output = Vec::new();
            let webp_encoder = WebPEncoder::new_lossless(&mut webp_output);
            
            if img.write_with_encoder(webp_encoder).is_ok() && !webp_output.is_empty() {
                if webp_output.len() < best_size {
                    best_output = webp_output;
                    best_size = best_output.len();
                }
            }
            
            // Strategy 2: Try RGB conversion if it has an alpha channel
            if let DynamicImage::ImageRgba8(rgba_img) = img {
                let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
                
                if !has_transparency {
                    // Convert to RGB and encode as WebP
                    let mut rgb_output = Vec::new();
                    let rgb_encoder = WebPEncoder::new_lossless(&mut rgb_output);
                    let rgb_img = img.to_rgb8();
                    
                    if rgb_img.write_with_encoder(rgb_encoder).is_ok() && rgb_output.len() < best_size {
                        best_output = rgb_output;
                        best_size = best_output.len();
                    }
                }
            }
            
            // Ensure we have actual compression
            if best_output.is_empty() {
                return Err(PixieError::ProcessingError(
                    "WebP re-encoding failed".to_string()
                ));
            }
            
            Ok(best_output)
        },
        
        WebPOptimizationStrategy::ConvertToJPEG { jpeg_quality } => {
            // Convert WebP to JPEG for better compatibility
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            // Convert to RGB before JPEG encoding
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("WebP to JPEG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        WebPOptimizationStrategy::ConvertToPNG => {
            // Convert to lossless PNG
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("WebP to PNG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        WebPOptimizationStrategy::MetadataStripping => {
            // First encode as PNG to get clean data, then apply metadata stripping logic
            let mut png_data = Vec::new();
            let png_encoder = image::codecs::png::PngEncoder::new(&mut png_data);
            img.write_with_encoder(png_encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("PNG encoding for metadata stripping failed: {}", e)
                ))?;
            
            // For WebP metadata stripping, we need the original WebP data
            // This strategy should be applied to the original data, not the decoded image
            Err(PixieError::ProcessingError("Metadata stripping requires original WebP data".to_string()))
        },
    }
}
/// Strip WebP metadata for optimization
fn strip_webp_metadata(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    result.extend_from_slice(&data[0..12]); // RIFF header + WEBP signature
    
    let mut pos = 12;
    
    // Parse WebP chunks and filter based on quality
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Determine which chunks to keep based on quality
        let keep_chunk = match chunk_id {
            b"VP8 " | b"VP8L" | b"VP8X" => true, // Core image data - always keep
            b"ANIM" | b"ANMF" => quality >= 90, // Animation - keep for high quality
            b"ICCP" | b"EXIF" => quality >= 80, // Color/metadata - medium quality
            b"XMP " => quality >= 70, // Extended metadata
            _ => quality >= 95, // Other chunks only at highest quality
        };
        
        if keep_chunk {
            // Copy chunk header and data
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
        }
        
        pos += 8 + chunk_size;
        
        // Align to even byte boundary (WebP requirement)
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    Ok(result)
}

/// Fallback when WebP codec is not available
#[cfg(not(feature = "codec-webp"))]
pub fn optimize_webp(_data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    Err(PixieError::FeatureNotAvailable("WebP codec not available - enable codec-webp feature".into()))
}

/// Optimize WebP image (main entry point)
pub fn optimize_webp(data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
    optimize_webp_rust(data, quality)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

/// Optimize WebP with configuration (alternative entry point)
pub fn optimize_webp_with_config_alt(data: &[u8], quality: u8, config: &ImageOptConfig) -> OptResult<Vec<u8>> {
    optimize_webp_with_config(data, quality, config)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

/// Alias for compatibility with existing code
pub fn optimize_webp_old(data: &[u8], quality: u8, config: &ImageOptConfig) -> OptResult<Vec<u8>> {
    let _ = config; // Ignore config for now
    optimize_webp(data, quality)
}

/// Check if data is valid WebP format
pub fn is_webp(data: &[u8]) -> bool {
    data.len() >= 12 && 
    data[0..4] == [0x52, 0x49, 0x46, 0x46] && // "RIFF"
    data[8..12] == [0x57, 0x45, 0x42, 0x50]    // "WEBP"
}

/// Get WebP image dimensions without full decode
pub fn get_webp_dimensions(data: &[u8]) -> PixieResult<(u32, u32)> {
    if !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid WebP file".into()));
    }
    
    use image::load_from_memory;
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    Ok((img.width(), img.height()))
}
