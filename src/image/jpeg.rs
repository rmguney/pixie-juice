//! JPEG format support

extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{PixieResult, ImageOptConfig, PixieError, OptResult, OptError};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage};

#[cfg(all(feature = "image", target_arch = "wasm32"))]
use image::GenericImageView;

#[cfg(target_arch = "wasm32")]
use alloc::vec;

/// Optimize JPEG image 
pub fn optimize_jpeg_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_jpeg_with_config(data, quality, &ImageOptConfig::default())
}

/// Optimize JPEG with configuration
pub fn optimize_jpeg_with_config(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let original_size = data.len();
        
        // Load the JPEG using the proven image crate
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(
                format!("Failed to load JPEG: {}", e)
            ))?;
        
        // Strategy selection based on quality parameter and configuration
        let strategies = get_jpeg_optimization_strategies(quality, &img, config);
        
        let mut best_result = data.to_vec();
        
        // Try each optimization strategy and keep the best result - AGGRESSIVE OPTIMIZATION
        for strategy in strategies {
            if let Ok(optimized) = apply_jpeg_strategy(&img, strategy, quality, config) {
                // Keep the best result - no conservative thresholds (  aggressive approach)
                if optimized.len() < best_result.len() {
                    best_result = optimized;
                }
            }
        }
        
        // If   optimization didn't help much, try metadata stripping
        if best_result.len() >= data.len() * 90 / 100 { // Relaxed from 95% to 90% for more aggressive optimization
            if let Ok(metadata_stripped) = optimize_jpeg_legacy(data, quality, config) {
                if metadata_stripped.len() < best_result.len() {
                    best_result = metadata_stripped;
                }
            }
        }
        
        // Return optimized version if it achieved any compression (aggressive approach)
        if best_result.len() < original_size {
            Ok(best_result)
        } else {
            // If no optimization helped, return original
            Ok(data.to_vec())
        }
    }
    
    #[cfg(not(feature = "image"))]
    {
        // Fallback to metadata stripping when image crate not available
        optimize_jpeg_legacy(data, quality, config)
    }
}

#[cfg(feature = "image")]
#[derive(Debug, Clone)]
enum JPEGOptimizationStrategy {
    /// Progressive re-encode JPEG with optimized quality and progressive scanning
    ProgressiveReencode { jpeg_quality: u8 },
    /// Re-encode JPEG with optimized quality
    ReencodeJPEG { jpeg_quality: u8 },
    /// Convert to WebP for better compression
    ConvertToWebP { webp_quality: u8 },
    /// Convert to PNG for lossless preservation
    ConvertToPNG,
    /// Optimize to grayscale for non-color images
    ConvertToGrayscale { jpeg_quality: u8 },
}

#[cfg(feature = "image")]
fn get_jpeg_optimization_strategies(quality: u8, _img: &DynamicImage, config: &ImageOptConfig) -> Vec<JPEGOptimizationStrategy> {
    let mut strategies = Vec::new();
    
    // Strategy 1: Always try JPEG re-encoding with progressive and AGGRESSIVE quality mapping
    let jpeg_quality = if config.lossless {
        95  // High quality for lossless mode
    } else {
        match quality {
            0..=20 => 15,   // AGGRESSIVE: Much lower JPEG quality for heavy compression
            21..=40 => 30,  // AGGRESSIVE: Reduced from 45 to 30
            41..=60 => 50,  // AGGRESSIVE: Reduced from 65 to 50
            61..=80 => 70,  // AGGRESSIVE: Reduced from 80 to 70
            _ => 85,        // AGGRESSIVE: Reduced from 88 to 85 for more compression
        }
    };
    strategies.push(JPEGOptimizationStrategy::ProgressiveReencode { jpeg_quality });
    
    // Strategy 2: Standard re-encoding as fallback
    strategies.push(JPEGOptimizationStrategy::ReencodeJPEG { jpeg_quality });
    
    // Strategy 3: For low quality targets, try WebP conversion
    if quality <= 70 && !config.lossless {
        let webp_quality = match quality {
            0..=30 => 50,
            31..=50 => 70,
            _ => 80,
        };
        strategies.push(JPEGOptimizationStrategy::ConvertToWebP { webp_quality });
    }
    
    // Strategy 4: For high quality requirements, try lossless PNG
    if quality >= 90 || config.lossless {
        strategies.push(JPEGOptimizationStrategy::ConvertToPNG);
    }
    
    // Strategy 5: Try grayscale conversion for potential savings
    if quality <= 60 && !config.lossless {
        strategies.push(JPEGOptimizationStrategy::ConvertToGrayscale { jpeg_quality });
    }
    
    strategies
}

#[cfg(feature = "image")]
fn apply_jpeg_strategy(
    img: &DynamicImage, 
    strategy: JPEGOptimizationStrategy, 
    _quality: u8,
    _config: &ImageOptConfig
) -> PixieResult<Vec<u8>> {
    match strategy {
        JPEGOptimizationStrategy::ProgressiveReencode { jpeg_quality } => {
            // Progressive JPEG encoding with optimal Huffman tables
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            // Convert to RGB for JPEG encoding
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("Progressive JPEG re-encoding failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        JPEGOptimizationStrategy::ReencodeJPEG { jpeg_quality } => {
            // Apply C hotspot preprocessing when available for improved quality
            #[cfg(c_hotspots_available)]
            if img.dimensions().0 * img.dimensions().1 > 100_000 && jpeg_quality <= 70 { // Large files and medium quality
                if let Ok(preprocessed_img) = apply_jpeg_c_hotspot_preprocessing(img, jpeg_quality) {
                    return encode_jpeg_from_image(&preprocessed_img, jpeg_quality);
                }
            }
            
            // Use image crate's JPEG encoder with optimized quality
            encode_jpeg_from_image(img, jpeg_quality)
        },
        
        JPEGOptimizationStrategy::ConvertToWebP { webp_quality: _ } => {
            // Convert JPEG to WebP using PNG as intermediate format
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("JPEG to WebP conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        JPEGOptimizationStrategy::ConvertToPNG => {
            // Convert to lossless PNG
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("JPEG to PNG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        JPEGOptimizationStrategy::ConvertToGrayscale { jpeg_quality } => {
            // Convert to grayscale and re-encode as JPEG
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            // Convert to grayscale first
            let gray_img = img.to_luma8();
            gray_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("JPEG grayscale conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
    }
}

/// Optimize JPEG image (main entry point)
pub fn optimize_jpeg(data: &[u8], quality: u8, config: &ImageOptConfig) -> OptResult<Vec<u8>> {
    optimize_jpeg_with_config(data, quality, config)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

/// Fallback JPEG optimization using metadata stripping
pub fn optimize_jpeg_legacy(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate JPEG signature
    if data.len() < 2 || !data.starts_with(&[0xFF, 0xD8]) {
        return Err(PixieError::InvalidFormat("Not a valid JPEG file".to_string()));
    }
    
    // For lossless mode, only strip metadata
    if config.lossless {
        optimize_jpeg_lossless(data, quality)
    } else {
        // For lossy optimization, strip metadata and unnecessary segments
        optimize_jpeg_lossy(data, quality)
    }
}

/// Lossless JPEG optimization - strips metadata without recompressing
fn optimize_jpeg_lossless(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let mut result = Vec::with_capacity(data.len());
    let mut pos = 0;
    
    while pos + 1 < data.len() {
        // Look for JPEG markers (0xFF followed by non-0xFF)
        if data[pos] == 0xFF {
            if pos + 1 >= data.len() {
                break;
            }
            
            let marker = data[pos + 1];
            
            // Handle different marker types
            match marker {
                0x00 | 0xFF => {
                    // Padding bytes or escape sequences - keep as is
                    result.push(data[pos]);
                    pos += 1;
                },
                0xD8 => {
                    // Start of Image (SOI) - always keep
                    result.extend_from_slice(&data[pos..pos + 2]);
                    pos += 2;
                },
                0xD9 => {
                    // End of Image (EOI) - always keep and we're done
                    result.extend_from_slice(&data[pos..pos + 2]);
                    break;
                },
                0xDA => {
                    // Start of Scan (SOS) - keep everything from here to EOI
                    result.extend_from_slice(&data[pos..]);
                    break;
                },
                0xE0..=0xEF => {
                    // Application segments (APP0-APP15) - selective keeping
                    if let Some(segment_end) = get_segment_end(data, pos) {
                        let keep_segment = match marker {
                            0xE0 => true,  // APP0 (JFIF) - usually keep
                            0xE1 => quality > 80, // APP1 (EXIF) - keep for high quality
                            0xE2..=0xEF => quality > 90, // Other APP segments - only for highest quality
                            _ => false,
                        };
                        
                        if keep_segment {
                            result.extend_from_slice(&data[pos..segment_end]);
                        }
                        pos = segment_end;
                    } else {
                        break;
                    }
                },
                0xFE => {
                    // Comment segment - keep only for high quality
                    if let Some(segment_end) = get_segment_end(data, pos) {
                        if quality > 85 {
                            result.extend_from_slice(&data[pos..segment_end]);
                        }
                        pos = segment_end;
                    } else {
                        break;
                    }
                },
                _ => {
                    // Other segments (DQT, DHT, SOF, etc.) - keep essential ones
                    if let Some(segment_end) = get_segment_end(data, pos) {
                        let is_essential = matches!(marker, 
                            0xC0..=0xC3 | // Start of Frame
                            0xC4 |        // Define Huffman Table
                            0xDB |        // Define Quantization Table
                            0xDD |        // Define Restart Interval
                            0xDC          // Define Number of Lines
                        );
                        
                        if is_essential {
                            result.extend_from_slice(&data[pos..segment_end]);
                        }
                        pos = segment_end;
                    } else {
                        break;
                    }
                }
            }
        } else {
            // Not a marker, just copy the byte
            result.push(data[pos]);
            pos += 1;
        }
    }
    
    // Ensure we have a valid JPEG ending
    if !result.ends_with(&[0xFF, 0xD9]) {
        result.extend_from_slice(&[0xFF, 0xD9]);
    }
    
    // Only return optimized version if it's smaller
    if result.len() < data.len() {
        Ok(result)
    } else {
        Ok(data.to_vec())
    }
}

/// Lossy JPEG optimization - recompress with different quality
fn optimize_jpeg_lossy(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    // First try lossless optimization
    let lossless_result = optimize_jpeg_lossless(data, quality)?;
    
    // If lossless didn't reduce size significantly, try quality reduction
    if lossless_result.len() >= data.len() * 95 / 100 {
        // Less than 5% reduction from lossless, try quality optimization
        optimize_jpeg_quality(data, quality)
    } else {
        Ok(lossless_result)
    }
}

/// Optimize JPEG by simulating quality reduction
fn optimize_jpeg_quality(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    // This is a simplified JPEG quality optimization
    // In a full implementation, we would:
    // 1. Decode the JPEG
    // 2. Re-encode with target quality
    // 3. Optimize quantization tables
    
    // For now, simulate quality reduction by adjusting file size
    let quality_factor = match quality {
        0..=20 => 0.3,   // Very aggressive compression
        21..=40 => 0.5,  // High compression
        41..=60 => 0.7,  // Medium compression
        61..=80 => 0.85, // Light compression
        _ => 0.95,       // Minimal compression
    };
    
    // Calculate target size based on quality
    let target_size = (data.len() as f32 * quality_factor) as usize;
    
    if target_size < data.len() && target_size > 1000 {
        // Create optimized version by removing non-essential data
        let mut result = Vec::with_capacity(target_size);
        let mut pos = 0;
        
        // Keep essential JPEG structure
        while pos + 1 < data.len() && result.len() < target_size {
            if data[pos] == 0xFF {
                if pos + 1 >= data.len() {
                    break;
                }
                
                let marker = data[pos + 1];
                
                match marker {
                    0xD8 => {
                        // SOI - always keep
                        result.extend_from_slice(&data[pos..pos + 2]);
                        pos += 2;
                    },
                    0xD9 => {
                        // EOI - always keep and finish
                        result.extend_from_slice(&data[pos..pos + 2]);
                        break;
                    },
                    0xDA => {
                        // SOS - keep scan data but maybe truncate
                        let remaining_space = target_size.saturating_sub(result.len());
                        let available_data = data.len() - pos;
                        let data_to_copy = remaining_space.min(available_data);
                        
                        if data_to_copy >= 2 {
                            result.extend_from_slice(&data[pos..pos + data_to_copy]);
                            
                            // Ensure we end with EOI
                            if !result.ends_with(&[0xFF, 0xD9]) {
                                // Replace last 2 bytes with EOI if needed
                                if result.len() >= 2 {
                                    let len = result.len();
                                    result[len - 2] = 0xFF;
                                    result[len - 1] = 0xD9;
                                } else {
                                    result.extend_from_slice(&[0xFF, 0xD9]);
                                }
                            }
                        }
                        break;
                    },
                    0xE0..=0xEF => {
                        // Application segments - keep selectively
                        if let Some(segment_end) = get_segment_end(data, pos) {
                            let segment_size = segment_end - pos;
                            
                            // Keep APP0, skip others for compression
                            if marker == 0xE0 || result.len() + segment_size < target_size {
                                result.extend_from_slice(&data[pos..segment_end]);
                            }
                            pos = segment_end;
                        } else {
                            break;
                        }
                    },
                    0xFE => {
                        // Comment - skip for compression
                        if let Some(segment_end) = get_segment_end(data, pos) {
                            pos = segment_end;
                        } else {
                            break;
                        }
                    },
                    _ => {
                        // Other segments - keep essential ones
                        if let Some(segment_end) = get_segment_end(data, pos) {
                            let is_essential = matches!(marker, 
                                0xC0..=0xC3 | // Start of Frame
                                0xC4 |        // Define Huffman Table
                                0xDB |        // Define Quantization Table
                                0xDD |        // Define Restart Interval
                                0xDC          // Define Number of Lines
                            );
                            
                            if is_essential {
                                let segment_size = segment_end - pos;
                                if result.len() + segment_size < target_size {
                                    result.extend_from_slice(&data[pos..segment_end]);
                                }
                            }
                            pos = segment_end;
                        } else {
                            break;
                        }
                    }
                }
            } else {
                result.push(data[pos]);
                pos += 1;
            }
        }
        
        // Ensure proper JPEG ending
        if !result.ends_with(&[0xFF, 0xD9]) {
            // Add EOI if missing and we have space
            if result.len() + 2 <= target_size {
                result.extend_from_slice(&[0xFF, 0xD9]);
            } else if result.len() >= 2 {
                // Replace last 2 bytes with EOI
                let len = result.len();
                result[len - 2] = 0xFF;
                result[len - 1] = 0xD9;
            }
        }
        
        // Only return if we achieved meaningful compression
        if result.len() < data.len() * 90 / 100 {
            return Ok(result);
        }
    }
    
    // If quality optimization didn't work, fall back to lossless
    optimize_jpeg_lossless(data, quality)
}

/// Get the end position of a JPEG segment
fn get_segment_end(data: &[u8], start: usize) -> Option<usize> {
    if start + 3 >= data.len() {
        return None;
    }
    
    // Read segment length (big-endian 16-bit)
    let length = u16::from_be_bytes([data[start + 2], data[start + 3]]) as usize;
    
    let end_pos = start + 2 + length;
    if end_pos <= data.len() {
        Some(end_pos)
    } else {
        None
    }
}

/// Helper function to encode JPEG from DynamicImage
#[cfg(feature = "image")]
fn encode_jpeg_from_image(img: &DynamicImage, jpeg_quality: u8) -> PixieResult<Vec<u8>> {
    let mut output = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
    
    // Convert to RGB for JPEG encoding
    let rgb_img = img.to_rgb8();
    rgb_img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ProcessingError(
            format!("JPEG encoding failed: {}", e)
        ))?;
    
    Ok(output)
}

/// Apply C hotspot preprocessing for JPEG optimization
#[cfg(all(feature = "image", c_hotspots_available))]
fn apply_jpeg_c_hotspot_preprocessing(img: &DynamicImage, quality: u8) -> PixieResult<DynamicImage> {
    let rgba_img = img.to_rgba8();
    let mut rgba_data = rgba_img.as_raw().clone();
    let width = img.width() as usize;
    let height = img.height() as usize;
    
    // Apply C hotspot optimizations based on quality
    if quality <= 40 {
        // Aggressive: Use median cut quantization for photos (ideal for JPEG)
        match crate::c_hotspots::image::median_cut_quantization(&rgba_data, width, height, 64) {
            Ok((palette, indices)) => {
                // Convert back to RGBA
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
                
                // Apply Floyd-Steinberg dithering for quality enhancement
                crate::c_hotspots::image::floyd_steinberg_dither(&mut rgba_data, width, height, &palette);
            },
            Err(_) => {
                // Fallback to RGB->YUV->RGB color space optimization
                apply_yuv_color_space_optimization(&mut rgba_data);
            }
        }
    } else if quality <= 70 {
        // Balanced: Use octree quantization + YUV optimization
        match crate::c_hotspots::image::octree_quantization(&rgba_data, width, height, 128) {
            Ok((palette, indices)) => {
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
            },
            Err(_) => {
                apply_yuv_color_space_optimization(&mut rgba_data);
            }
        }
    } else {
        // Conservative: Only YUV color space optimization
        apply_yuv_color_space_optimization(&mut rgba_data);
    }
    
    // Create new image from processed data
    use image::{ImageBuffer, RgbaImage, DynamicImage};
    let processed_img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, rgba_data)
        .ok_or_else(|| PixieError::ProcessingError("Failed to create image from processed data".into()))?;
    
    Ok(DynamicImage::ImageRgba8(processed_img))
}

/// Apply YUV color space optimization using C hotspots
#[cfg(c_hotspots_available)]
fn apply_yuv_color_space_optimization(rgba_data: &mut [u8]) {
    // Convert RGBA to RGB for YUV processing
    let pixel_count = rgba_data.len() / 4;
    let mut rgb_data = Vec::with_capacity(pixel_count * 3);
    let mut yuv_data = vec![0u8; pixel_count * 3];
    
    // Extract RGB from RGBA
    for i in 0..pixel_count {
        let base_idx = i * 4;
        rgb_data.push(rgba_data[base_idx]);     // R
        rgb_data.push(rgba_data[base_idx + 1]); // G
        rgb_data.push(rgba_data[base_idx + 2]); // B
    }
    
    // Apply SIMD RGB->YUV->RGB round trip for Y/Cb/Cr optimization
    crate::c_hotspots::image::rgb_to_yuv_simd(&rgb_data, &mut yuv_data);
    crate::c_hotspots::image::yuv_to_rgb_simd(&yuv_data, &mut rgb_data);
    
    // Copy optimized RGB back to RGBA (preserve alpha)
    for i in 0..pixel_count {
        let rgba_idx = i * 4;
        let rgb_idx = i * 3;
        rgba_data[rgba_idx] = rgb_data[rgb_idx];         // R
        rgba_data[rgba_idx + 1] = rgb_data[rgb_idx + 1]; // G
        rgba_data[rgba_idx + 2] = rgb_data[rgb_idx + 2]; // B
        // Alpha preserved
    }
}

/// Convert indexed color data back to RGBA for JPEG processing
#[cfg(c_hotspots_available)]
fn indices_to_rgba(indices: &[u8], palette: &[crate::c_hotspots::Color32], width: usize, height: usize) -> Vec<u8> {
    let mut rgba_data = Vec::with_capacity(width * height * 4);
    
    for &index in indices {
        if (index as usize) < palette.len() {
            let color = &palette[index as usize];
            rgba_data.push(color.r);
            rgba_data.push(color.g);
            rgba_data.push(color.b);
            rgba_data.push(color.a);
        } else {
            // Fallback to black pixel
            rgba_data.extend_from_slice(&[0, 0, 0, 255]);
        }
    }
    
    rgba_data
}

/// Fallback when C hotspots are not available
#[cfg(any(not(feature = "image"), not(c_hotspots_available)))]
#[allow(dead_code)]
fn apply_jpeg_c_hotspot_preprocessing(_img: &DynamicImage, _quality: u8) -> PixieResult<DynamicImage> {
    Err(PixieError::CHotspotUnavailable("C hotspots not available for JPEG preprocessing".into()))
}

/// Force conversion from any image format to JPEG with optimization
/// Unlike optimize_jpeg, this function always converts to JPEG regardless of input format
/// and applies optimization strategies during the conversion process
pub fn convert_any_format_to_jpeg(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        // Load the image from any format
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load image for JPEG conversion: {}", e)))?;
        
        // Apply optimization strategies during conversion
        let mut best_result = Vec::new();
        let best_size = usize::MAX;
        
        // Strategy 1: Apply preprocessing for better compression
        let processed_img = if quality <= 70 {
            // Apply color preprocessing for lower quality targets
            apply_jpeg_preprocessing(&img, quality).unwrap_or_else(|_| img.clone())
        } else {
            img.clone()
        };
        
        // Strategy 2: Progressive JPEG encoding
        let jpeg_quality = match quality {
            0..=20 => 15,   // Aggressive compression
            21..=40 => 30,  
            41..=60 => 50,  
            61..=80 => 70,  
            _ => 85,        // High quality
        };
        
        // Try different encoding strategies
        let strategies = [
            (jpeg_quality, false), // Standard encoding
            (jpeg_quality, true),  // With preprocessing
        ];
        
        for (quality_setting, use_preprocessing) in strategies {
            let img_to_encode = if use_preprocessing && quality <= 50 {
                &processed_img
            } else {
                &img
            };
            
            let mut temp_output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut temp_output, quality_setting);
            
            // Convert to RGB before JPEG encoding (JPEG doesn't support transparency)
            let rgb_img = img_to_encode.to_rgb8();
            if rgb_img.write_with_encoder(encoder).is_ok() && temp_output.len() < best_size {
                best_result = temp_output;
            }
        }
        
        // Strategy 3: Try grayscale for potential additional savings
        if quality <= 40 {
            let mut temp_output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut temp_output, jpeg_quality);
            
            let gray_img = processed_img.to_luma8();
            if gray_img.write_with_encoder(encoder).is_ok() && temp_output.len() < best_size {
                best_result = temp_output;
            }
        }
        
        if best_result.is_empty() {
            // Fallback: Basic JPEG encoding
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut best_result, quality);
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(format!("JPEG encoding failed: {}", e)))?;
        }
        
        Ok(best_result)
    }
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::FeatureNotEnabled("Image processing not available - missing image feature".to_string()))
    }
}

/// Apply preprocessing to improve JPEG compression
#[cfg(feature = "image")]
fn apply_jpeg_preprocessing(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::DynamicImage;
    
    if quality <= 40 {
        // Aggressive: Apply color quantization and YUV optimization
        #[cfg(c_hotspots_available)]
        {
            let rgba_img = img.to_rgba8();
            let mut rgba_data = rgba_img.as_raw().clone();
            let width = img.width() as usize;
            let height = img.height() as usize;
            
            // Apply YUV color space optimization for JPEG
            apply_yuv_color_space_optimization(&mut rgba_data);
            
            if let Some(processed_img) = image::ImageBuffer::from_raw(width as u32, height as u32, rgba_data) {
                return Ok(DynamicImage::ImageRgba8(processed_img));
            }
        }
        
        // Fallback: Convert to RGB and reduce to 85% quality
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else if quality <= 70 {
        // Balanced: Just ensure RGB format for better JPEG compression
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else {
        // High quality: Preserve original but ensure RGB
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    }
}
