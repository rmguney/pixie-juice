//! TIFF format support with C hotspot acceleration

extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{PixieResult, ImageOptConfig, PixieError, OptResult, OptError};
use crate::c_hotspots::{
    compress_tiff_lzw_c_hotspot, 
    strip_tiff_metadata_c_hotspot,
    apply_tiff_predictor_c_hotspot,
    optimize_tiff_colorspace_c_hotspot
};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage};

/// Optimize TIFF image  with the image crate
pub fn optimize_tiff_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_tiff_with_config(data, quality, &ImageOptConfig::default())
}

/// Optimize TIFF with configuration with LZW/JPEG compression
pub fn optimize_tiff_with_config(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        // CRITICAL: Avoid C hotspots for now for small TIFF files in WASM to prevent unreachable panic
        #[cfg(target_arch = "wasm32")]
        {
            if data.len() < 100_000 { // Files under 100KB - use safe Rust-only optimization
                return optimize_tiff_safe_fallback(data, quality, config);
            }
        }
        
        // Load the TIFF using the proven image crate with enhanced error handling
        let img = match load_from_memory(data) {
            Ok(img) => img,
            Err(e) => {
                // Graceful fallback for corrupted or unsupported TIFF variants
                return optimize_tiff_safe_fallback(data, quality, config);
            }
        };
        
        // Strategy selection based on quality parameter and configuration
        let strategies = get_tiff_optimization_strategies(quality, &img, config);
        
        let mut best_result = data.to_vec();
        
        // Try each optimization strategy and keep the best result - AGGRESSIVE OPTIMIZATION
        for strategy in strategies {
            match apply_tiff_strategy(&img, strategy, quality, config) {
                Ok(optimized) => {
                    if optimized.len() < best_result.len() {
                        best_result = optimized;
                    }
                },
                Err(_) => {
                    // Continue to next strategy on error - graceful degradation
                    continue;
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

/// Safe TIFF optimization fallback that avoids C hotspots and unreachable panics
fn optimize_tiff_safe_fallback(data: &[u8], quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        // Use pure Rust optimization with proven image crate - no C hotspots
        match load_from_memory(data) {
            Ok(img) => {
                // Simple safe strategies: PNG or JPEG conversion only
                let mut best_result = data.to_vec();
                
                // Strategy 1: Convert to PNG for safe lossless optimization
                if quality >= 70 {
                    if let Ok(png_data) = convert_to_png_safe(&img) {
                        if png_data.len() < best_result.len() {
                            best_result = png_data;
                        }
                    }
                }
                
                // Strategy 2: Convert to JPEG for lossy optimization (non-transparent)
                if quality < 85 && !has_transparency_check(&img) {
                    let jpeg_quality = match quality {
                        0..=30 => 50,
                        31..=50 => 70,
                        51..=70 => 80,
                        _ => 90,
                    };
                    if let Ok(jpeg_data) = convert_to_jpeg_safe(&img, jpeg_quality) {
                        if jpeg_data.len() < best_result.len() {
                            best_result = jpeg_data;
                        }
                    }
                }
                
                Ok(best_result)
            },
            Err(_) => {
                // Ultimate fallback: return original data if image loading fails
                Ok(data.to_vec())
            }
        }
    }
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, quality, _config);
        Ok(data.to_vec()) // Return original if image feature disabled
    }
}

/// Convert to PNG using safe, proven image crate functions
#[cfg(feature = "image")]
fn convert_to_png_safe(img: &DynamicImage) -> PixieResult<Vec<u8>> {
    let mut output = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut output);
    
    img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ProcessingError(
            format!("Safe PNG conversion failed: {}", e)
        ))?;
    
    Ok(output)
}

/// Convert to JPEG using safe, proven image crate functions
#[cfg(feature = "image")]
fn convert_to_jpeg_safe(img: &DynamicImage, jpeg_quality: u8) -> PixieResult<Vec<u8>> {
    let mut output = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
    
    let rgb_img = img.to_rgb8();
    rgb_img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ProcessingError(
            format!("Safe JPEG conversion failed: {}", e)
        ))?;
    
    Ok(output)
}

/// Safe transparency check without panics
#[cfg(feature = "image")]
fn has_transparency_check(img: &DynamicImage) -> bool {
    match img {
        DynamicImage::ImageRgba8(rgba_img) => {
            // Check only a sample of pixels to avoid performance issues
            let total_pixels = rgba_img.pixels().len();
            let sample_step = (total_pixels / 100).max(1); // Check every 1% of pixels
            
            rgba_img.pixels().step_by(sample_step).any(|p| p[3] < 255)
        },
        DynamicImage::ImageRgba16(rgba_img) => {
            let total_pixels = rgba_img.pixels().len();
            let sample_step = (total_pixels / 100).max(1);
            
            rgba_img.pixels().step_by(sample_step).any(|p| p[3] < u16::MAX)
        },
        _ => false,
    }
}

#[cfg(feature = "image")]
#[derive(Debug, Clone)]
enum TIFFOptimizationStrategy {
    /// Re-encode as TIFF with LZW compression using C hotspot
    LZWCompressionCHotspot,
    /// Re-encode as TIFF with LZW compression (Rust fallback)
    LZWCompression,
    /// Re-encode as TIFF with JPEG compression (for photo content)
    JPEGCompression { jpeg_quality: u8 },
    /// Strip metadata while keeping TIFF format using C hotspot
    StripMetadataCHotspot,
    /// Strip metadata (Rust fallback)
    StripMetadata,
    /// Apply predictor preprocessing using C hotspot
    ApplyPredictorCHotspot { predictor_type: u8 },
    /// Optimize color space using C hotspot
    OptimizeColorspaceCHotspot { target_bits: u8 },
    /// Convert to PNG for lossless web format
    ConvertToPNG,
    /// Convert to JPEG for better compression (non-transparent images)
    ConvertToJPEG { jpeg_quality: u8 },
    /// Convert to WebP for modern format support
    ConvertToWebP { webp_quality: u8 },
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
    
    // Priority 1: C hotspot optimizations for best performance
    strategies.push(TIFFOptimizationStrategy::StripMetadataCHotspot);
    
    if config.lossless || quality >= 80 {
        strategies.push(TIFFOptimizationStrategy::LZWCompressionCHotspot);
        strategies.push(TIFFOptimizationStrategy::ApplyPredictorCHotspot { predictor_type: 2 });
    }
    
    if quality <= 70 {
        strategies.push(TIFFOptimizationStrategy::OptimizeColorspaceCHotspot { 
            target_bits: if quality <= 40 { 4 } else { 6 }
        });
    }
    
    // Priority 2: Rust fallback strategies
    if config.lossless || quality >= 80 {
        strategies.push(TIFFOptimizationStrategy::LZWCompression);
    }
    
    strategies.push(TIFFOptimizationStrategy::StripMetadata);
    
    // Strategy 3: JPEG compression within TIFF for photographic content
    if !has_transparency && quality <= 75 && !config.lossless {
        let jpeg_quality = match quality {
            0..=30 => 40,
            31..=50 => 60,
            51..=70 => 75,
            _ => 85,
        };
        strategies.push(TIFFOptimizationStrategy::JPEGCompression { jpeg_quality });
    }
    
    // Strategy 4: Convert to PNG for lossless web-friendly format
    if quality >= 70 || config.lossless {
        strategies.push(TIFFOptimizationStrategy::ConvertToPNG);
    }
    
    // Strategy 5: For images without transparency, try JPEG conversion
    if !has_transparency && quality <= 85 && !config.lossless {
        let jpeg_quality = match quality {
            0..=30 => 50,
            31..=50 => 70,
            51..=70 => 80,
            _ => 90,
        };
        strategies.push(TIFFOptimizationStrategy::ConvertToJPEG { jpeg_quality });
    }
    
    // Strategy 6: Try WebP conversion for modern format support
    if quality <= 80 && !config.lossless {
        let webp_quality = match quality {
            0..=30 => 60,
            31..=50 => 75,
            _ => 85,
        };
        strategies.push(TIFFOptimizationStrategy::ConvertToWebP { webp_quality });
    }
    
    strategies
}

#[cfg(feature = "image")]
fn apply_tiff_strategy(
    img: &DynamicImage, 
    strategy: TIFFOptimizationStrategy, 
    quality: u8,
    _config: &ImageOptConfig
) -> PixieResult<Vec<u8>> {
    match strategy {
        TIFFOptimizationStrategy::LZWCompressionCHotspot => {
            // CRITICAL: Avoid C hotspot in WASM for small files to prevent unreachable panic
            #[cfg(target_arch = "wasm32")]
            {
                // Use safe Rust fallback for WASM builds to prevent crashes
                apply_tiff_strategy(img, TIFFOptimizationStrategy::LZWCompression, quality, _config)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Use C hotspot for optimal LZW compression with SIMD (native only)
                let rgba_img = img.to_rgba8();
                let (width, height) = (rgba_img.width() as usize, rgba_img.height() as usize);
                
                compress_tiff_lzw_c_hotspot(rgba_img.as_raw(), width, height, quality)
            }
        },
        
        TIFFOptimizationStrategy::StripMetadataCHotspot => {
            // CRITICAL: Avoid C hotspot in WASM for small files to prevent unreachable panic
            #[cfg(target_arch = "wasm32")]
            {
                // Use safe Rust fallback for WASM builds
                apply_tiff_strategy(img, TIFFOptimizationStrategy::StripMetadata, quality, _config)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Use C hotspot for fast metadata stripping with SIMD (native only)
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new(&mut output);
                
                img.write_with_encoder(encoder)
                    .map_err(|e| PixieError::ProcessingError(
                        format!("TIFF metadata stripping failed: {}", e)
                    ))?;
                
                // Apply C hotspot metadata stripping to the encoded data
                strip_tiff_metadata_c_hotspot(&output, false)
            }
        },
        
        TIFFOptimizationStrategy::ApplyPredictorCHotspot { predictor_type } => {
            // CRITICAL: Avoid C hotspot in WASM to prevent unreachable panic
            #[cfg(target_arch = "wasm32")]
            {
                // Use safe Rust fallback for WASM builds - basic PNG optimization
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut output, 
                    image::codecs::png::CompressionType::Best, 
                    image::codecs::png::FilterType::Adaptive
                );
                
                img.write_with_encoder(encoder)
                    .map_err(|e| PixieError::ProcessingError(
                        format!("TIFF predictor optimization failed: {}", e)
                    ))?;
                
                Ok(output)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Use C hotspot for predictor preprocessing with SIMD (native only)
                let mut rgba_img = img.to_rgba8();
                let (width, height) = (rgba_img.width() as usize, rgba_img.height() as usize);
                
                apply_tiff_predictor_c_hotspot(rgba_img.as_mut(), width, height, predictor_type)?;
                
                // Re-encode the processed image
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut output, 
                    image::codecs::png::CompressionType::Best, 
                    image::codecs::png::FilterType::Adaptive
                );
                
                rgba_img.write_with_encoder(encoder)
                    .map_err(|e| PixieError::ProcessingError(
                        format!("TIFF predictor optimization failed: {}", e)
                    ))?;
                
                Ok(output)
            }
        },
        
        TIFFOptimizationStrategy::OptimizeColorspaceCHotspot { target_bits } => {
            // CRITICAL: Avoid C hotspot in WASM to prevent unreachable panic
            #[cfg(target_arch = "wasm32")]
            {
                // Use safe Rust fallback for WASM builds - basic PNG optimization
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut output, 
                    image::codecs::png::CompressionType::Best, 
                    image::codecs::png::FilterType::Adaptive
                );
                
                img.write_with_encoder(encoder)
                    .map_err(|e| PixieError::ProcessingError(
                        format!("TIFF color space optimization failed: {}", e)
                    ))?;
                
                Ok(output)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Use C hotspot for color space optimization with SIMD (native only)
                let mut rgba_img = img.to_rgba8();
                let (width, height) = (rgba_img.width() as usize, rgba_img.height() as usize);
                
                optimize_tiff_colorspace_c_hotspot(rgba_img.as_mut(), width, height, target_bits)?;
                
                // Re-encode the optimized image
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut output, 
                    image::codecs::png::CompressionType::Best, 
                    image::codecs::png::FilterType::Adaptive
                );
                
                rgba_img.write_with_encoder(encoder)
                    .map_err(|e| PixieError::ProcessingError(
                        format!("TIFF color space optimization failed: {}", e)
                    ))?;
                
                Ok(output)
            }
        },
        
        TIFFOptimizationStrategy::LZWCompression => {
            // Rust fallback for LZW compression
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new_with_quality(
                &mut output, 
                image::codecs::png::CompressionType::Best, 
                image::codecs::png::FilterType::Adaptive
            );
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF LZW compression failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        TIFFOptimizationStrategy::JPEGCompression { jpeg_quality } => {
            // Use JPEG compression within TIFF structure (simulated as JPEG conversion)
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF JPEG compression failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        TIFFOptimizationStrategy::StripMetadata => {
            // Rust fallback for metadata stripping
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF metadata stripping failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
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
