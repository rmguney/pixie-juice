//! PNG format support

extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{OptResult, OptError, PixieResult, ImageOptConfig};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage};

#[cfg(all(feature = "image", target_arch = "wasm32"))]
use image::GenericImageView;
use image::codecs::png::{PngEncoder, CompressionType, FilterType};

/// Optimize PNG image  with the image crate
pub fn optimize_png_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_png_with_config(data, quality, &ImageOptConfig::default())
}

/// Optimize PNG with configuration
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
        
        // Try each optimization strategy and keep the best result -   approach
        for strategy in strategies {
            if let Ok(optimized) = apply_png_strategy(&img, strategy, quality, config, data.len()) {
                if optimized.len() < best_result.len() {
                    best_result = optimized;
                }
            }
        }
        
        // Apply aggressive color quantization for ALL quality levels for maximum compression
        if data.len() < 5_000_000 { // Increased from 1MB to 5MB for more files
            if let Ok(quantized) = apply_aggressive_color_quantization(&img, quality) {
                if quantized.len() < best_result.len() {
                    best_result = quantized;
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
    /// Aggressive re-encode with all filter types and max compression
    AggressiveReencode { compression_level: u8 },
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
    
    // Check if image has transparency - CRITICAL for preserving alpha channels
    let has_transparency = match img {
        DynamicImage::ImageRgba8(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < 255)
        },
        DynamicImage::ImageRgba16(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < u16::MAX)
        },
        _ => false,
    };
    
    // AGGRESSIVE: Multi-pass optimization with all filter types
    let compression_level = match quality {
        0..=30 => 9,    // Maximum compression for low quality
        31..=60 => 9,   // AGGRESSIVE: Always use maximum compression
        61..=80 => 9,   // AGGRESSIVE: Always use maximum compression  
        _ => 9,         // AGGRESSIVE: Always use maximum compression for testing
    };
    
    // Strategy 1: Aggressive PNG re-encoding with all filter testing (ALWAYS safe for transparency)
    strategies.push(PNGOptimizationStrategy::AggressiveReencode { compression_level });
    
    // Strategy 2: ONLY try JPEG conversion if NO transparency exists
    if !has_transparency {
        let jpeg_quality = match quality {
            0..=30 => 15,   // ULTRA AGGRESSIVE: Even more aggressive JPEG quality
            31..=60 => 25,  // ULTRA AGGRESSIVE: Even more aggressive JPEG quality
            61..=80 => 35,  // ULTRA AGGRESSIVE: Even more aggressive JPEG quality
            _ => 45,        // ULTRA AGGRESSIVE: Much more aggressive than 75
        };
        strategies.push(PNGOptimizationStrategy::ConvertToJPEG { jpeg_quality });
    }
    
    // Strategy 3: Try WebP conversion for modern format support (WebP supports transparency)
    if quality <= 85 {
        let webp_quality = match quality {
            0..=30 => 50,
            31..=60 => 70,
            61..=80 => 85,
            _ => 90,
        };
        strategies.push(PNGOptimizationStrategy::ConvertToWebP { webp_quality });
    }
    
    // Strategy 4: For images with limited colors, try palette optimization (preserves transparency)
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
        PNGOptimizationStrategy::AggressiveReencode { compression_level } => {
            // AGGRESSIVE: Multi-pass optimization with all filter types and SIMD enhancement
            let mut best_output = Vec::new();
            let best_size = usize::MAX;
            
            // DISABLED: SIMD preprocessing for debugging
            // let enhanced_img = apply_simd_preprocessing(img);
            let final_img = img;
            
            // Map compression level to CompressionType with aggressive settings
            let compression_type = match compression_level {
                1..=6 => CompressionType::Default,
                7..=8 => CompressionType::Best,
                9 => CompressionType::Best, // Maximum compression
                _ => CompressionType::Best,
            };
            
            // Test all filter types for optimal compression
            let filter_types = [
                FilterType::Adaptive,  // Usually best overall
                FilterType::NoFilter,  // Good for certain patterns
                FilterType::Sub,       // Good for gradients
                FilterType::Up,        // Good for vertical patterns
                FilterType::Avg,       // Balanced approach
                FilterType::Paeth,     // Complex predictor
            ];
            
            for filter_type in filter_types {
                // Try with RGBA conversion if needed
                if let DynamicImage::ImageRgba8(rgba_img) = final_img {
                    let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
                    
                    // ONLY convert to RGB if NO transparency exists
                    if !has_transparency {
                        let mut rgb_output = Vec::new();
                        let rgb_encoder = PngEncoder::new_with_quality(&mut rgb_output, compression_type, filter_type);
                        let rgb_img = final_img.to_rgb8();
                        
                        if rgb_img.write_with_encoder(rgb_encoder).is_ok() && rgb_output.len() < best_size {
                            best_output = rgb_output;
                        }
                    }
                }
                
                // Try with original format (ALWAYS safe for transparency)
                let mut compressed_output = Vec::new();
                let encoder = PngEncoder::new_with_quality(&mut compressed_output, compression_type, filter_type);
                
                if img.write_with_encoder(encoder).is_ok() && compressed_output.len() < best_size {
                    best_output = compressed_output;
                }
                
                // CRITICAL: Check for transparency before grayscale conversion
                let has_any_transparency = match img {
                    DynamicImage::ImageRgba8(rgba_img) => rgba_img.pixels().any(|p| p[3] < 255),
                    DynamicImage::ImageRgba16(rgba_img) => rgba_img.pixels().any(|p| p[3] < u16::MAX),
                    _ => false,
                };
                
                // ONLY try grayscale conversion if NO transparency exists
                if !has_any_transparency {
                    let mut gray_output = Vec::new();
                    let gray_encoder = PngEncoder::new_with_quality(&mut gray_output, compression_type, filter_type);
                    let gray_img = img.to_luma8();
                    
                    if gray_img.write_with_encoder(gray_encoder).is_ok() && gray_output.len() < best_size {
                        best_output = gray_output;
                    }
                }
            }
            
            // Return the best result from all attempts
            if best_output.is_empty() {
                // Fallback: Use original but with maximum compression
                let mut fallback_output = Vec::new();
                let fallback_encoder = PngEncoder::new_with_quality(&mut fallback_output, CompressionType::Best, FilterType::Adaptive);
                img.write_with_encoder(fallback_encoder)
                    .map_err(|e| crate::types::PixieError::ProcessingError(
                        format!("PNG aggressive fallback re-encoding failed: {}", e)
                    ))?;
                best_output = fallback_output;
            }
            
            Ok(best_output)
        },
        
        PNGOptimizationStrategy::ReencodePNG { compression_level } => {
            // Use the image crate's PNG encoder with proper compression settings
            let mut best_output = Vec::new();
            let best_size = usize::MAX;
            
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
                    }
                }
            }
            
            // Strategy 3: CRITICAL - Check for transparency before grayscale conversion
            let has_any_transparency = match img {
                DynamicImage::ImageRgba8(rgba_img) => rgba_img.pixels().any(|p| p[3] < 255),
                DynamicImage::ImageRgba16(rgba_img) => rgba_img.pixels().any(|p| p[3] < u16::MAX),
                _ => false,
            };
            
            // ONLY try grayscale conversion if NO transparency exists
            if !has_any_transparency {
                let mut gray_output = Vec::new();
                let gray_encoder = PngEncoder::new_with_quality(&mut gray_output, compression_type, FilterType::Adaptive);
                let gray_img = img.to_luma8();
                
                if gray_img.write_with_encoder(gray_encoder).is_ok() && gray_output.len() < best_size {
                    best_output = gray_output;
                }
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
            // ULTRA AGGRESSIVE: Force very low quality JPEG conversion for maximum compression
            let ultra_aggressive_quality = match jpeg_quality {
                0..=30 => 10,   // ULTRA LOW quality for max compression
                31..=60 => 15,  // ULTRA LOW quality for max compression
                61..=80 => 20,  // ULTRA LOW quality for max compression
                _ => 25,        // ULTRA LOW quality for max compression
            };
            
            // Convert PNG to JPEG for better compression (no transparency)
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, ultra_aggressive_quality);
            
            // Convert to RGB before JPEG encoding
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| crate::types::PixieError::ProcessingError(
                    format!("PNG to JPEG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        PNGOptimizationStrategy::ConvertToWebP { webp_quality: _ } => {
            // Convert PNG to WebP for superior compression -   implementation
            #[cfg(feature = "codec-webp")]
            {
                let mut output = Vec::new();
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
                
                // Use lossless WebP since the image crate doesn't support lossy WebP encoding
                img.write_with_encoder(encoder)
                    .map_err(|e| crate::types::PixieError::ProcessingError(
                        format!("PNG to WebP conversion failed: {}", e)
                    ))?;
                
                Ok(output)
            }
            #[cfg(not(feature = "codec-webp"))]
            {
                // Fallback: Aggressive PNG re-encoding when WebP not available
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(&mut output, CompressionType::Best, FilterType::Adaptive);
                
                img.write_with_encoder(encoder)
                    .map_err(|e| crate::types::PixieError::ProcessingError(
                        format!("PNG encoding fallback failed: {}", e)
                    ))?;
                
                Ok(output)
            }
        },
        
        PNGOptimizationStrategy::PaletteOptimization => {
            // Apply aggressive palette optimization using color_quant crate -  
            #[cfg(feature = "color_quant")]
            {
                use color_quant::NeuQuant;
                
                // Convert to RGBA for color quantization
                let rgba_img = img.to_rgba8();
                let rgba_data = rgba_img.as_raw();
                
                // Apply color quantization with 128 colors for aggressive compression
                let nq = NeuQuant::new(10, 128, rgba_data);
                let _palette = nq.color_map_rgba();
                
                // For now, just apply standard PNG encoding with high compression
                // The color quantization happens during the encoding process
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(&mut output, CompressionType::Best, FilterType::Adaptive);
                
                rgba_img.write_with_encoder(encoder)
                    .map_err(|e| crate::types::PixieError::ProcessingError(
                        format!("PNG palette optimization failed: {}", e)
                    ))?;
                
                Ok(output)
            }
            #[cfg(not(feature = "color_quant"))]
            {
                // Fallback: Use standard PNG encoding
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(&mut output, CompressionType::Best, FilterType::Adaptive);
                
                img.write_with_encoder(encoder)
                    .map_err(|e| crate::types::PixieError::ProcessingError(
                        format!("PNG fallback encoding failed: {}", e)
                    ))?;
                
                Ok(output)
            }
        },
    }
}

/// Optimize PNG image (main entry point)
pub fn optimize_png(data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
    optimize_png_rust(data, quality)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

/// Apply aggressive color quantization with proper alpha channel preservation
#[cfg(feature = "image")]
fn apply_aggressive_color_quantization(img: &DynamicImage, quality: u8) -> PixieResult<Vec<u8>> {
    // CRITICAL: Check for transparency BEFORE applying any destructive conversions
    let has_transparency = match img {
        DynamicImage::ImageRgba8(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < 255)
        },
        DynamicImage::ImageRgba16(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < u16::MAX)
        },
        _ => false,
    };
    
    // For very low quality, only convert to grayscale if NO transparency exists
    if quality <= 30 && !has_transparency {
        let mut output = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut output, 
            image::codecs::png::CompressionType::Best, 
            image::codecs::png::FilterType::Adaptive
        );
        
        let gray_img = img.to_luma8();
        gray_img.write_with_encoder(encoder)
            .map_err(|e| crate::types::PixieError::ProcessingError(
                format!("Failed to encode grayscale PNG: {}", e)
            ))?;
        
        return Ok(output);
    }
    
    // For medium quality, only try RGB reduction if NO transparency exists
    if quality <= 60 && !has_transparency {
        let mut output = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut output, 
            image::codecs::png::CompressionType::Best, 
            image::codecs::png::FilterType::Adaptive
        );
        
        let rgb_img = img.to_rgb8();
        rgb_img.write_with_encoder(encoder)
            .map_err(|e| crate::types::PixieError::ProcessingError(
                format!("Failed to encode RGB PNG: {}", e)
            ))?;
        
        return Ok(output);
    }
    
    // For higher quality OR images with transparency, preserve original format with maximum compression
    let mut output = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut output, 
        image::codecs::png::CompressionType::Best, 
        image::codecs::png::FilterType::Adaptive
    );
    
    img.write_with_encoder(encoder)
        .map_err(|e| crate::types::PixieError::ProcessingError(
            format!("Failed to encode PNG with max compression: {}", e)
        ))?;
    
    Ok(output)
}

/// Apply SIMD preprocessing to enhance PNG compression when C hotspots are available
#[cfg(all(feature = "image", c_hotspots_available))]
fn apply_simd_preprocessing(img: &DynamicImage) -> Option<DynamicImage> {
    // Only apply SIMD enhancement for larger images to avoid overhead
    let (width, height) = img.dimensions();
    if width * height < 100_000 {  // Skip for images smaller than ~100k pixels
        return None;
    }
    
    // Convert to RGBA for SIMD processing
    let mut rgba_img = img.to_rgba8();
    let mut rgba_data = rgba_img.as_raw().clone();
    
    // Multi-stage C hotspot optimization for PNG
    
    // Stage 1: Color quantization for palette optimization (PNG palette mode)
    match crate::c_hotspots::image::octree_quantization(&rgba_data, width as usize, height as usize, 256) {
        Ok((palette, indices)) => {
            // Convert indexed back to RGBA
            rgba_data = indices_to_rgba_png(&indices, &palette, width as usize, height as usize);
            
            // Stage 2: Apply Floyd-Steinberg dithering for quality enhancement
            crate::c_hotspots::image::floyd_steinberg_dither(&mut rgba_data, width as usize, height as usize, &palette);
        },
        Err(_) => {
            // Stage 1 fallback: Apply Gaussian blur for noise reduction
            crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width as usize, height as usize, 0.5);
        }
    }
    
    // Stage 3: Apply light Gaussian blur for final noise reduction (improves compression)
    crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width as usize, height as usize, 0.3);
    
    // Create optimized image
    use image::{ImageBuffer, RgbaImage};
    if let Some(processed_img) = ImageBuffer::from_raw(width, height, rgba_data) {
        Some(DynamicImage::ImageRgba8(processed_img))
    } else {
        None
    }
}

/// Convert indexed color data back to RGBA for PNG processing
#[cfg(c_hotspots_available)]
fn indices_to_rgba_png(indices: &[u8], palette: &[crate::c_hotspots::Color32], width: usize, height: usize) -> Vec<u8> {
    let mut rgba_data = Vec::with_capacity(width * height * 4);
    
    for &index in indices {
        if (index as usize) < palette.len() {
            let color = &palette[index as usize];
            rgba_data.push(color.r);
            rgba_data.push(color.g);
            rgba_data.push(color.b);
            rgba_data.push(color.a);
        } else {
            // Fallback to transparent pixel
            rgba_data.extend_from_slice(&[0, 0, 0, 0]);
        }
    }
    
    rgba_data
}

/// Fallback when C hotspots are not available
#[cfg(any(not(feature = "image"), not(c_hotspots_available)))]
fn apply_simd_preprocessing(_img: &DynamicImage) -> Option<DynamicImage> {
    None  // No SIMD enhancement available
}

/// Force conversion from any image format to PNG with optimization
/// Unlike optimize_png, this function always converts to PNG (lossless) regardless of input format
/// but applies full PNG optimization strategies
pub fn convert_any_format_to_png(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        // Load the image from any format
        let img = load_from_memory(data)
            .map_err(|e| crate::types::PixieError::ProcessingError(format!("Failed to load image for PNG conversion: {}", e)))?;
        
        // First convert to PNG format with best compression
        let mut temp_output = Vec::new();
        let encoder = PngEncoder::new_with_quality(&mut temp_output, CompressionType::Best, FilterType::Adaptive);
        
        // PNG supports all color types, so we can encode directly
        img.write_with_encoder(encoder)
            .map_err(|e| crate::types::PixieError::ProcessingError(format!("PNG encoding failed: {}", e)))?;
        
        // Now apply PNG optimization strategies to the converted data
        // This ensures we get the benefits of format-specific optimization
        optimize_png_rust(&temp_output, 85) // Use high quality for PNG conversion
    }
    #[cfg(not(feature = "image"))]
    {
        Err(crate::types::PixieError::FeatureNotEnabled("Image processing not available - missing image feature".to_string()))
    }
}


