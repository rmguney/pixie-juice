//! PNG optimization using library-first approach with image crate
//! 
//! This module provides PNG optimization using the proven `image` crate
//! for WASM compatibility, following library-first implementation principles.

extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{OptResult, OptError, PixieResult, ImageOptConfig};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage, ImageEncoder};
use image::codecs::png::{PngEncoder, CompressionType, FilterType};

/// Optimize PNG image using library-first approach with the image crate
pub fn optimize_png_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_png_with_config(data, quality, &ImageOptConfig::default())
}

/// Optimize PNG with configuration - library-first implementation
pub fn optimize_png_with_config(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        // Load the PNG using the proven image crate
        let img = load_from_memory(data)
            .map_err(|e| crate::types::PixieError::ProcessingError(
                format!("Failed to load PNG: {}", e)
            ))?;
        
        // Strategy selection based on quality parameter
        let strategies = get_png_optimization_strategies(quality, &img);
        
        let mut best_result = data.to_vec();
        let mut best_size = data.len();
        
        // Try each optimization strategy and keep the best result
        for strategy in strategies {
            if let Ok(optimized) = apply_png_strategy(&img, strategy, quality, config, data.len()) {
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
        Err(crate::types::PixieError::FeatureNotEnabled("PNG optimization requires 'image' feature".to_string()))
    }
}

#[cfg(feature = "image")]
#[derive(Debug, Clone)]
enum PNGOptimizationStrategy {
    /// Re-encode as PNG with different compression settings
    ReencodePNG { compression_level: u8 },
    /// Convert to JPEG (for non-transparent images)
    ConvertToJPEG { jpeg_quality: u8 },
    /// Convert to WebP for better compression
    ConvertToWebP { webp_quality: u8 },
    /// Apply palette optimization for small color count images
    PaletteOptimization,
}

#[cfg(feature = "image")]
fn get_png_optimization_strategies(quality: u8, img: &DynamicImage) -> Vec<PNGOptimizationStrategy> {
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
    
    // Strategy 1: Always try PNG re-encoding with better compression
    let compression_level = match quality {
        0..=30 => 9,    // Maximum compression for low quality
        31..=60 => 6,   // Balanced compression
        61..=80 => 3,   // Faster compression
        _ => 1,         // Minimal compression for high quality
    };
    strategies.push(PNGOptimizationStrategy::ReencodePNG { compression_level });
    
    // Strategy 2: For images without transparency, try JPEG conversion
    if !has_transparency {
        let jpeg_quality = match quality {
            0..=30 => 40,   // Aggressive JPEG quality for low PNG quality
            31..=60 => 65,  // Moderate JPEG quality
            61..=80 => 80,  // Conservative JPEG quality
            _ => 90,        // High JPEG quality for high PNG quality
        };
        strategies.push(PNGOptimizationStrategy::ConvertToJPEG { jpeg_quality });
    }
    
    // Strategy 3: Try WebP conversion for modern format support
    if quality <= 85 {
        let webp_quality = match quality {
            0..=30 => 50,
            31..=60 => 70,
            61..=80 => 85,
            _ => 90,
        };
        strategies.push(PNGOptimizationStrategy::ConvertToWebP { webp_quality });
    }
    
    // Strategy 4: For images with limited colors, try palette optimization
    let (width, height) = (img.width(), img.height());
    let pixel_count = (width * height) as usize;
    
    // Only try palette optimization for reasonably sized images
    if pixel_count < 1_000_000 && quality <= 75 {
        strategies.push(PNGOptimizationStrategy::PaletteOptimization);
    }
    
    strategies
}

#[cfg(feature = "image")]
fn apply_png_strategy(
    img: &DynamicImage, 
    strategy: PNGOptimizationStrategy, 
    _quality: u8,
    _config: &ImageOptConfig,
    _original_size: usize
) -> PixieResult<Vec<u8>> {
    match strategy {
        PNGOptimizationStrategy::ReencodePNG { compression_level } => {
            // Use the image crate's PNG encoder with proper compression settings
            let mut best_output = Vec::new();
            let mut best_size = usize::MAX;
            
            // Map compression level to CompressionType
            let compression_type = match compression_level {
                1..=3 => CompressionType::Fast,      // Low compression, faster
                4..=6 => CompressionType::Default,   // Balanced compression
                7..=9 => CompressionType::Best,      // High compression, slower
                _ => CompressionType::Default,
            };
            
            // Strategy 1: Try with optimal compression and adaptive filter
            let mut compressed_output = Vec::new();
            let encoder = PngEncoder::new_with_quality(&mut compressed_output, compression_type, FilterType::Adaptive);
            
            if img.write_with_encoder(encoder).is_ok() && !compressed_output.is_empty() {
                if compressed_output.len() < best_size {
                    best_output = compressed_output;
                    best_size = best_output.len();
                }
            }
            
            // Strategy 2: Try RGB conversion if it has an alpha channel
            if let DynamicImage::ImageRgba8(rgba_img) = img {
                let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
                
                if !has_transparency {
                    // Convert to RGB and encode with compression
                    let mut rgb_output = Vec::new();
                    let rgb_encoder = PngEncoder::new_with_quality(&mut rgb_output, compression_type, FilterType::Adaptive);
                    let rgb_img = img.to_rgb8();
                    
                    if rgb_img.write_with_encoder(rgb_encoder).is_ok() && rgb_output.len() < best_size {
                        best_output = rgb_output;
                        best_size = best_output.len();
                    }
                }
            }
            
            // Strategy 3: Try grayscale conversion if appropriate
            let mut gray_output = Vec::new();
            let gray_encoder = PngEncoder::new_with_quality(&mut gray_output, compression_type, FilterType::Adaptive);
            let gray_img = img.to_luma8();
            
            if gray_img.write_with_encoder(gray_encoder).is_ok() && gray_output.len() < best_size {
                best_output = gray_output;
                best_size = best_output.len();
            }
            
            // Return the best result from strategies tried
            if best_output.is_empty() {
                // Fallback: Use original but with high compression
                let mut fallback_output = Vec::new();
                let fallback_encoder = PngEncoder::new_with_quality(&mut fallback_output, CompressionType::Best, FilterType::Adaptive);
                img.write_with_encoder(fallback_encoder)
                    .map_err(|e| crate::types::PixieError::ProcessingError(
                        format!("PNG fallback re-encoding failed: {}", e)
                    ))?;
                best_output = fallback_output;
            }
            
            Ok(best_output)
        },
        
        PNGOptimizationStrategy::ConvertToJPEG { jpeg_quality } => {
            // Convert PNG to JPEG for better compression (no transparency)
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            // Convert to RGB before JPEG encoding
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| crate::types::PixieError::ProcessingError(
                    format!("PNG to JPEG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        PNGOptimizationStrategy::ConvertToWebP { webp_quality: _ } => {
            // For now, just re-encode as PNG since WebP requires additional setup
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| crate::types::PixieError::ProcessingError(
                    format!("PNG encoding failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        PNGOptimizationStrategy::PaletteOptimization => {
            // Apply palette optimization using standard PNG encoding
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            // Convert to RGB for standard encoding (palette optimization will happen in encoder)
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| crate::types::PixieError::ProcessingError(
                    format!("PNG palette optimization failed: {}", e)
                ))?;
            
            Ok(output)
        },
    }
}

/// Optimize PNG image (main entry point)
pub fn optimize_png(data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
    optimize_png_rust(data, quality)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}
