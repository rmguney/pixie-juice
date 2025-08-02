//! TIFF optimization using library-first approach with image crate
//! 
//! This module provides TIFF optimization using the proven `image` crate
//! for WASM compatibility, following library-first implementation principles.

extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{PixieResult, ImageOptConfig, PixieError, OptResult, OptError};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage};

/// Optimize TIFF image using library-first approach with the image crate
pub fn optimize_tiff_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_tiff_with_config(data, quality, &ImageOptConfig::default())
}

/// Optimize TIFF with configuration - library-first implementation
pub fn optimize_tiff_with_config(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        // Load the TIFF using the proven image crate
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(
                format!("Failed to load TIFF: {}", e)
            ))?;
        
        // Strategy selection based on quality parameter and configuration
        let strategies = get_tiff_optimization_strategies(quality, &img, config);
        
        let mut best_result = data.to_vec();
        let mut best_size = data.len();
        
        // Try each optimization strategy and keep the best result
        for strategy in strategies {
            if let Ok(optimized) = apply_tiff_strategy(&img, strategy, quality, config) {
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
            // If no optimization helped, return original
            Ok(data.to_vec())
        }
    }
    
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, quality, config);
        Err(PixieError::FeatureNotEnabled("TIFF optimization requires 'image' feature".to_string()))
    }
}

#[cfg(feature = "image")]
#[derive(Debug, Clone)]
enum TIFFOptimizationStrategy {
    /// Convert to PNG for lossless web format
    ConvertToPNG,
    /// Convert to JPEG for better compression (non-transparent images)
    ConvertToJPEG { jpeg_quality: u8 },
    /// Convert to WebP for modern format support
    ConvertToWebP { webp_quality: u8 },
    /// Re-encode as TIFF with different compression
    ReencodeTIFF,
}

#[cfg(feature = "image")]
fn get_tiff_optimization_strategies(quality: u8, img: &DynamicImage, config: &ImageOptConfig) -> Vec<TIFFOptimizationStrategy> {
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
    
    // Strategy 1: Convert to PNG for lossless web-friendly format
    if quality >= 70 || config.lossless {
        strategies.push(TIFFOptimizationStrategy::ConvertToPNG);
    }
    
    // Strategy 2: For images without transparency, try JPEG conversion
    if !has_transparency && quality <= 85 && !config.lossless {
        let jpeg_quality = match quality {
            0..=30 => 50,
            31..=50 => 70,
            51..=70 => 80,
            _ => 90,
        };
        strategies.push(TIFFOptimizationStrategy::ConvertToJPEG { jpeg_quality });
    }
    
    // Strategy 3: Try WebP conversion for modern format support
    if quality <= 80 && !config.lossless {
        let webp_quality = match quality {
            0..=30 => 60,
            31..=50 => 75,
            _ => 85,
        };
        strategies.push(TIFFOptimizationStrategy::ConvertToWebP { webp_quality });
    }
    
    // Strategy 4: Try TIFF re-encoding (fallback)
    strategies.push(TIFFOptimizationStrategy::ReencodeTIFF);
    
    strategies
}

#[cfg(feature = "image")]
fn apply_tiff_strategy(
    img: &DynamicImage, 
    strategy: TIFFOptimizationStrategy, 
    _quality: u8,
    _config: &ImageOptConfig
) -> PixieResult<Vec<u8>> {
    match strategy {
        TIFFOptimizationStrategy::ConvertToPNG => {
            // Convert TIFF to PNG for web compatibility
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF to PNG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        TIFFOptimizationStrategy::ConvertToJPEG { jpeg_quality } => {
            // Convert TIFF to JPEG for better compression
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            // Convert to RGB before JPEG encoding
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF to JPEG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        TIFFOptimizationStrategy::ConvertToWebP { webp_quality: _ } => {
            // Convert to PNG as intermediate format (WebP would need additional setup)
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF to WebP conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        TIFFOptimizationStrategy::ReencodeTIFF => {
            // Re-encode as PNG since TIFF encoding may not be available in image crate
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF re-encoding failed: {}", e)
                ))?;
            
            Ok(output)
        },
    }
}

/// Check if data is valid TIFF format
pub fn is_tiff(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    
    // TIFF: "II" (little-endian) or "MM" (big-endian) followed by 42
    (data[0..2] == [0x49, 0x49] && data[2] == 42 && data[3] == 0) ||  // II42 (little-endian)
    (data[0..2] == [0x4D, 0x4D] && data[2] == 0 && data[3] == 42)     // MM42 (big-endian)
}

/// Optimize TIFF image (main entry point)
pub fn optimize_tiff(data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
    optimize_tiff_rust(data, quality)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}
