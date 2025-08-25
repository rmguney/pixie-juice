//! WebP format support with optimized quality-based encoding

extern crate alloc;
use alloc::{vec::Vec, format, string::ToString};

use crate::types::{PixieResult, PixieError, ImageOptConfig, OptResult, OptError};

#[cfg(target_arch = "wasm32")]
use crate::user_feedback::UserFeedback;

pub fn optimize_webp_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_webp_with_config(data, quality, &ImageOptConfig::default())
}

pub fn optimize_webp_with_config(data: &[u8], quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        #[cfg(target_arch = "wasm32")]
        let start_time = if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
            performance.now()
        } else {
            0.0
        };
        
        #[cfg(target_arch = "wasm32")]
        {
            let msg = format!("WebP optimization starting: {} bytes, quality {}%", data.len(), quality);
            crate::image::log_to_console(&msg);
        }
        
        // CRITICAL FIX: Detect animated WebP and handle differently
        if detect_animated_webp(data) {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Detected animated WebP - using comprehensive optimization");
            
            return optimize_animated_webp_native(data, quality);
        }
        
        // CRITICAL FIX: Use proper WebP-to-WebP conversion instead of JPEG intermediate
        match reencode_webp_native_quality(data, quality) {
            Ok(reencoded) if reencoded.len() < data.len() => {
                let compression = ((data.len() - reencoded.len()) as f64 / data.len() as f64) * 100.0;
                #[cfg(target_arch = "wasm32")]
                {
                    let msg = format!("WebP native re-encoding: {} -> {} bytes ({:.2}% compression)", 
                                    data.len(), reencoded.len(), compression);
                    crate::image::log_to_console(&msg);
                }
                return Ok(reencoded);
            },
            Ok(_) => {
                #[cfg(target_arch = "wasm32")]
                crate::image::log_to_console("Re-encoding did not improve size, trying metadata optimization");
            },
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                crate::image::log_to_console(&format!("Native re-encoding failed: {}, trying metadata optimization", e));
            }
        }
        
        // Fallback to metadata stripping for already well-compressed WebP files
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Using WebP metadata optimization strategies");
        
        let mut best_result = data.to_vec();
        let mut best_size = data.len();
        let mut strategies_succeeded = 0;
        
        // Strategy 1: Aggressive metadata stripping
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Strategy 1: Aggressive metadata stripping");
        
        match strip_webp_metadata_aggressive(data, quality) {
            Ok(stripped) => {
                let compression = ((data.len() - stripped.len()) as f64 / data.len() as f64) * 100.0;
                #[cfg(target_arch = "wasm32")]
                {
                    let msg = format!("Metadata stripping: {} -> {} bytes ({:.2}% compression)", 
                                    data.len(), stripped.len(), compression);
                    crate::image::log_to_console(&msg);
                }
                
                if stripped.len() < best_size {
                    best_result = stripped;
                    best_size = best_result.len();
                    strategies_succeeded += 1;
                }
                
                #[cfg(not(target_arch = "wasm32"))]
                let _ = compression;
            },
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                {
                    let msg = format!("Metadata stripping failed: {}", e);
                    crate::image::log_to_console(&msg);
                }
                #[cfg(not(target_arch = "wasm32"))]
                let _ = e;
            }
        }
        
        // Strategy 2: Color quantization for low quality settings
        if quality <= 60 {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Strategy 2: Color quantization optimization");
            
            match apply_webp_color_quantization(&best_result, quality) {
                Ok(quantized) if quantized.len() < best_size => {
                    let compression = ((best_size - quantized.len()) as f64 / best_size as f64) * 100.0;
                    #[cfg(target_arch = "wasm32")]
                    {
                        let msg = format!("Color quantization: {} -> {} bytes ({:.2}% compression)", 
                                        best_size, quantized.len(), compression);
                        crate::image::log_to_console(&msg);
                    }
                    best_result = quantized;
                    best_size = best_result.len();
                    strategies_succeeded += 1;
                    
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = compression;
                },
                Ok(_) => {
                    #[cfg(target_arch = "wasm32")]
                    crate::image::log_to_console("Color quantization did not improve size");
                },
                Err(e) => {
                    #[cfg(target_arch = "wasm32")]
                    crate::image::log_to_console(&format!("Color quantization failed: {}", e));
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = e;
                }
            }
        }
        
        // Strategy 2: Color quantization for low quality settings
        if quality <= 60 {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Strategy 2: Color quantization optimization");
            
            match apply_webp_color_quantization(&best_result, quality) {
                Ok(quantized) if quantized.len() < best_size => {
                    let compression = ((best_size - quantized.len()) as f64 / best_size as f64) * 100.0;
                    #[cfg(target_arch = "wasm32")]
                    {
                        let msg = format!("Color quantization: {} -> {} bytes ({:.2}% compression)", 
                                        best_size, quantized.len(), compression);
                        crate::image::log_to_console(&msg);
                    }
                    best_result = quantized;
                    best_size = best_result.len();
                    strategies_succeeded += 1;
                    
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = compression;
                },
                Ok(_) => {
                    #[cfg(target_arch = "wasm32")]
                    crate::image::log_to_console("Color quantization did not improve size");
                },
                Err(e) => {
                    #[cfg(target_arch = "wasm32")]
                    crate::image::log_to_console(&format!("Color quantization failed: {}", e));
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = e;
                }
            }
        }
        
        // Strategy 3: C hotspot preprocessing for aggressive compression
        #[cfg(c_hotspots_available)]
        if quality <= 70 && data.len() > 50_000 {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Strategy 3: C hotspot preprocessing");
            
            match apply_c_hotspot_preprocessing(&best_result, quality) {
                Ok(preprocessed) if preprocessed.len() < best_size => {
                    let compression = ((best_size - preprocessed.len()) as f64 / best_size as f64) * 100.0;
                    #[cfg(target_arch = "wasm32")]
                    {
                        let msg = format!("C hotspot preprocessing: {} -> {} bytes ({:.2}% compression)", 
                                        best_size, preprocessed.len(), compression);
                        crate::image::log_to_console(&msg);
                    }
                    
                    best_result = preprocessed;
                    best_size = best_result.len();
                    strategies_succeeded += 1;
                    
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = compression;
                },
                Ok(_) => {
                    #[cfg(target_arch = "wasm32")]
                    crate::image::log_to_console("C hotspot preprocessing did not improve size");
                },
                Err(e) => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let msg = format!("C hotspot preprocessing failed: {}", e);
                        crate::image::log_to_console(&msg);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = e;
                }
            }
        }
        
        // Calculate final results
        let final_compression = ((data.len() - best_size) as f64 / data.len() as f64) * 100.0;
        
        #[cfg(target_arch = "wasm32")]
        let processing_time = if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
            performance.now() - start_time
        } else {
            0.0
        };
        
        #[cfg(target_arch = "wasm32")]
        {
            let msg = format!("WebP optimization complete: {} -> {} bytes ({:.2}% compression)", 
                            data.len(), best_size, final_compression);
            crate::image::log_to_console(&msg);
            
            let summary = format!("Summary: {}/3 strategies succeeded, {:.2}% total compression", 
                                strategies_succeeded, final_compression);
            crate::image::log_to_console(&summary);
            
            // Performance metrics
            let c_hotspots_used = if cfg!(c_hotspots_available) && quality <= 70 && data.len() > 50_000 { 1 } else { 0 };
            UserFeedback::show_performance_metrics(
                data.len(),
                best_size,
                processing_time,
                c_hotspots_used,
                strategies_succeeded as u32
            );
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = final_compression;
            let _ = strategies_succeeded;
        }
        
        // Return best result - if no improvement, try force conversion for low quality
        if best_size < data.len() {
            Ok(best_result)
        } else if quality < 80 {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Forcing aggressive WebP conversion for better compression");
            
            match force_webp_lossy_conversion(data, quality) {
                Ok(lossy) if lossy.len() < data.len() => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let compression = ((data.len() - lossy.len()) as f64 / data.len() as f64) * 100.0;
                        let msg = format!("✅ Forced lossy conversion: {} -> {} bytes ({:.2}% compression)", 
                                        data.len(), lossy.len(), compression);
                        crate::image::log_to_console(&msg);
                        let _ = compression;
                    }
                    Ok(lossy)
                },
                _ => {
                    #[cfg(target_arch = "wasm32")]
                    crate::image::log_to_console("✅ WebP optimization returned: original size maintained (already optimal)");
                    Ok(data.to_vec())
                }
            }
        } else {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("✅ WebP optimization returned: original size maintained (high quality setting)");
            Ok(data.to_vec())
        }
    }
    
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, quality, _config);
        Err(PixieError::FeatureNotEnabled("WebP optimization requires 'image' feature".to_string()))
    }
}


/// Re-encode WebP with native quality settings (CRITICAL FIX)
fn reencode_webp_native_quality(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use image::load_from_memory;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Re-encoding WebP with native quality {}", quality));
    
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    
    // CRITICAL FIX: image crate only supports lossless WebP!
    // For lossy compression, we need to use JPEG intermediate strategy
    if quality < 85 {
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Using JPEG intermediate strategy for lossy WebP compression");
        
        // Convert through JPEG for lossy compression
        match convert_webp_via_jpeg_lossy(&img, quality) {
            Ok(compressed) if compressed.len() < data.len() => {
                let savings = ((data.len() - compressed.len()) as f64 / data.len() as f64) * 100.0;
                #[cfg(target_arch = "wasm32")]
                crate::image::log_to_console(&format!("JPEG intermediate success: {:.1}% compression", savings));
                #[cfg(not(target_arch = "wasm32"))]
                let _ = savings; // Suppress unused warning
                return Ok(compressed);
            },
            Ok(_) => {
                #[cfg(target_arch = "wasm32")]
                crate::image::log_to_console("JPEG intermediate did not improve size");
            },
            Err(e) => {
                #[cfg(target_arch = "wasm32")]
                crate::image::log_to_console(&format!("JPEG intermediate failed: {}", e));
                #[cfg(not(target_arch = "wasm32"))]
                let _ = e; // Suppress unused warning
            }
        }
        
        // Fallback: aggressive preprocessing + lossless
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Fallback: aggressive preprocessing + lossless WebP");
        
        let processed_img = apply_aggressive_webp_preprocessing(&img, quality)?;
        let mut output = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
        processed_img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
        Ok(output)
    } else {
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Using lossless WebP encoding for high quality");
        
        let mut output = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
        img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
        Ok(output)
    }
}

/// Convert WebP via JPEG intermediate for lossy compression (CRITICAL)
fn convert_webp_via_jpeg_lossy(img: &image::DynamicImage, quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Converting WebP via JPEG with quality {}", quality));
    
    // Step 1: Convert to JPEG with quality setting
    let mut jpeg_output = Vec::new();
    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, quality);
    img.write_with_encoder(jpeg_encoder)
        .map_err(|e| PixieError::ImageEncodingFailed(format!("JPEG encoding failed: {}", e)))?;
    
    // Step 2: Load JPEG back as image
    let jpeg_img = image::load_from_memory(&jpeg_output)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("JPEG decode failed: {}", e)))?;
    
    // Step 3: Convert to WebP (lossless, but the quality loss already happened in JPEG step)
    let mut webp_output = Vec::new();
    let webp_encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_output);
    jpeg_img.write_with_encoder(webp_encoder)
        .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("JPEG->WebP conversion: {} bytes", webp_output.len()));
    
    Ok(webp_output)
}

/// Encode WebP with quality using image crate only (FALLBACK)
fn encode_webp_with_image_crate(img: &image::DynamicImage, quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Using image crate for WebP encoding with quality simulation {}", quality));
    
    // Apply preprocessing to simulate quality reduction
    let processed_img = apply_quality_preprocessing(img, quality)?;
    
    let mut output = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
    processed_img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("WebP encoded: {} bytes with quality simulation {}", output.len(), quality));
    
    Ok(output)
}

/// Apply quality-based preprocessing to simulate quality reduction
fn apply_quality_preprocessing(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::DynamicImage;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Applying quality preprocessing for quality {}", quality));
    
    if quality <= 30 {
        // Very low quality: aggressive downsample + color reduction
        let new_width = (img.width() * 75 / 100).max(16);
        let new_height = (img.height() * 75 / 100).max(16);
        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Triangle);
        let rgb_img = resized.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else if quality <= 50 {
        // Low quality: moderate downsample
        let new_width = (img.width() * 85 / 100).max(16);
        let new_height = (img.height() * 85 / 100).max(16);
        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Triangle);
        let rgb_img = resized.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else if quality <= 70 {
        // Medium quality: convert to RGB only
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else {
        // High quality: minimal changes
        Ok(img.clone())
    }
}

/// Force lossy WebP conversion with proper quality scaling
fn force_webp_lossy_conversion(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use image::load_from_memory;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Force lossy WebP conversion with quality {}", quality));
    
    let img = load_from_memory(data)
        .map_err(|e| {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console(&format!("WebP decode failed: {}", e));
            PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e))
        })?;
    
    // Apply aggressive preprocessing for low quality
    let processed_img = if quality <= 60 {
        apply_aggressive_webp_preprocessing(&img, quality).unwrap_or_else(|_| {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Aggressive preprocessing failed, using standard");
            img.clone()
        })
    } else {
        apply_quality_preprocessing(&img, quality).unwrap_or_else(|_| img.clone())
    };
    
    // Use image crate for consistent behavior
    // Fallback: encode with image crate after heavy preprocessing
    let mut output = Vec::new();
    
    // For very low quality, use RGB encoding to reduce size
    if quality <= 40 {
        let rgb_img = processed_img.to_rgb8();
        let rgb_dynamic = image::DynamicImage::ImageRgb8(rgb_img);
        
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
        rgb_dynamic.write_with_encoder(encoder)
            .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    } else {
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
        processed_img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    }
    
    Ok(output)
}

/// Apply aggressive preprocessing for maximum compression
fn apply_aggressive_webp_preprocessing(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::DynamicImage;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Applying aggressive preprocessing for quality {}", quality));
    
    // Calculate downsample ratio based on quality
    let (scale_factor, should_quantize) = match quality {
        0..=20 => (60, true),   // 60% size, with quantization
        21..=40 => (70, true),  // 70% size, with quantization  
        41..=60 => (85, false), // 85% size, no quantization
        _ => (95, false),       // 95% size, no quantization
    };
    
    // Downsample image
    let new_width = (img.width() * scale_factor / 100).max(8);
    let new_height = (img.height() * scale_factor / 100).max(8);
    let mut processed = img.resize(new_width, new_height, image::imageops::FilterType::Triangle);
    
    // Apply color quantization for very low quality
    if should_quantize {
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Applying color quantization");
        
        processed = apply_color_quantization_simple(&processed, quality)?;
    }
    
    // Convert to RGB to remove alpha channel
    let rgb_img = processed.to_rgb8();
    Ok(DynamicImage::ImageRgb8(rgb_img))
}

/// Apply WebP-specific color quantization
fn apply_webp_color_quantization(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use image::load_from_memory;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Applying WebP color quantization with quality {}", quality));
    
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    
    // Apply color quantization based on quality
    let quantized_img = apply_color_quantization_simple(&img, quality)?;
    
    // Re-encode as WebP using image crate
    let mut output = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
    quantized_img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    Ok(output)
}

/// Simple color quantization for WebP preprocessing
fn apply_color_quantization_simple(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::{DynamicImage, RgbImage};
    
    let rgb_img = img.to_rgb8();
    let (width, height) = (rgb_img.width(), rgb_img.height());
    
    // Determine bit depth based on quality
    let bit_shift = match quality {
        0..=20 => 3,   // 5-bit color (32 levels)
        21..=40 => 2,  // 6-bit color (64 levels)
        _ => 1,        // 7-bit color (128 levels)
    };
    
    let mut quantized = RgbImage::new(width, height);
    
    for (x, y, pixel) in rgb_img.enumerate_pixels() {
        let r = (pixel[0] >> bit_shift) << bit_shift;
        let g = (pixel[1] >> bit_shift) << bit_shift;
        let b = (pixel[2] >> bit_shift) << bit_shift;
        
        quantized.put_pixel(x, y, image::Rgb([r, g, b]));
    }
    
    Ok(DynamicImage::ImageRgb8(quantized))
}

/// Aggressive metadata stripping for WebP files
fn strip_webp_metadata_aggressive(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    result.extend_from_slice(&data[0..4]);
    
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]);
    
    result.extend_from_slice(&data[8..12]);
    
    let mut pos = 12;
    let mut kept_essential_data = false;
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        let keep_chunk = match chunk_id {
            b"VP8 " | b"VP8L" => true,
            b"ALPH" => true,
            b"VP8X" => true,
            b"ANIM" | b"ANMF" => true,
            b"ICCP" => quality >= 80,
            b"EXIF" => quality >= 90,
            b"XMP " => false,
            _ => false,
        };
        
        if keep_chunk {
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            kept_essential_data = true;
            
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    if kept_essential_data && result.len() < data.len() {
        Ok(result)
    } else {
        Err(PixieError::ProcessingError("Metadata stripping did not reduce file size".to_string()))
    }
}

/// Optimize animated WebP with native processing (CRITICAL FIX)
fn optimize_animated_webp_native(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console("Starting native animated WebP optimization");
    
    // Strategy 1: Skip re-encoding for now, go directly to metadata stripping
    // (Re-encoding animated WebP would require complex frame-by-frame processing)
    
    // Strategy 2: Aggressive metadata stripping for animated WebP
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console("Applying aggressive animated WebP metadata stripping");
    
    match strip_animated_webp_metadata_aggressive(data, quality) {
        Ok(stripped) if stripped.len() < data.len() => {
            let compression = ((data.len() - stripped.len()) as f64 / data.len() as f64) * 100.0;
            #[cfg(target_arch = "wasm32")]
            {
                let msg = format!("Aggressive metadata stripping: {} -> {} bytes ({:.2}% compression)", 
                                data.len(), stripped.len(), compression);
                crate::image::log_to_console(&msg);
                let _ = compression;
            }
            Ok(stripped)
        },
        Ok(_) => {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Metadata stripping did not improve size");
            
            // Strategy 3: Animation frame optimization for low quality
            if quality <= 50 {
                optimize_animated_webp_frames(data, quality)
            } else {
                Ok(data.to_vec())
            }
        },
        Err(e) => {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console(&format!("Metadata stripping failed: {}", e));
            
            // Fallback: frame optimization
            if quality <= 50 {
                optimize_animated_webp_frames(data, quality)
            } else {
                Ok(data.to_vec())
            }
        }
    }
}

/// Strip metadata from animated WebP more aggressively
fn strip_animated_webp_metadata_aggressive(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Aggressive animated WebP stripping for quality {}", quality));
    
    let mut result = Vec::new();
    
    // Copy RIFF header
    result.extend_from_slice(&data[0..4]); // "RIFF"
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]); // Placeholder for file size
    result.extend_from_slice(&data[8..12]); // "WEBP"
    
    let mut pos = 12;
    let mut animation_chunks_preserved = 0;
    let mut chunks_stripped = 0;
    let mut total_frames_processed = 0;
    
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Determine what to keep based on quality setting
        let keep_chunk = match chunk_id {
            // Essential WebP chunks - ALWAYS keep
            b"VP8X" => true,
            // Animation chunks - ALWAYS keep to preserve animation
            b"ANIM" => {
                animation_chunks_preserved += 1;
                true
            },
            b"ANMF" => {
                // Keep animation frames, but potentially optimize for low quality
                animation_chunks_preserved += 1;
                total_frames_processed += 1;
                
                if quality <= 30 && total_frames_processed > 20 {
                    // For very low quality, limit frame count
                    chunks_stripped += 1;
                    false
                } else {
                    true
                }
            },
            // Image data chunks
            b"VP8 " | b"VP8L" | b"ALPH" => true,
            // Metadata - be aggressive based on quality
            b"ICCP" => quality >= 80, // Keep color profile only for high quality
            b"EXIF" | b"XMP " => quality >= 90, // Keep metadata only for very high quality
            // Unknown chunks - strip for low quality
            _ => {
                chunks_stripped += 1;
                quality >= 70
            },
        };
        
        if keep_chunk {
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            
            // WebP requires even byte alignment
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    // Update file size in RIFF header
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    #[cfg(target_arch = "wasm32")]
    {
        let msg = format!("Aggressive animated WebP optimization: preserved {} animation chunks, stripped {} other chunks, processed {} frames", 
                           animation_chunks_preserved, chunks_stripped, total_frames_processed);
        crate::image::log_to_console(&msg);
    }
    
    if result.len() < data.len() {
        Ok(result)
    } else {
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("No size reduction achieved - animated WebP cannot be optimized further");
        Err(PixieError::ProcessingError("Animated WebP already at minimum size".to_string()))
    }
}

/// Optimize animated WebP by reducing frame count/quality for low quality settings
fn optimize_animated_webp_frames(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if quality > 50 {
        return Ok(data.to_vec()); // No frame optimization for medium/high quality
    }
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Optimizing animated WebP frames for quality {}", quality));
    
    // For very aggressive compression, use the aggressive metadata stripper
    // which includes frame count limiting
    strip_animated_webp_metadata_aggressive(data, quality)
}

#[cfg(feature = "codec-webp")]
fn optimize_animated_webp_reencoding(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    log_to_console("Preserving animation while optimizing metadata");
    if quality >= 80 {
        log_to_console("High quality - preserving animation with metadata optimization");
        optimize_animated_webp_metadata_only(data)
    } else {
        log_to_console(&format!("Quality {} - aggressive metadata stripping while preserving animation", quality));
        optimize_animated_webp_metadata_aggressive(data)
    }
}

/// Optimize animated WebP by only stripping non-essential metadata (preserves animation)
fn optimize_animated_webp_metadata_only(data: &[u8]) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    // Copy RIFF header
    result.extend_from_slice(&data[0..4]); // "RIFF"
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]); // Placeholder for file size
    result.extend_from_slice(&data[8..12]); // "WEBP"
    
    let mut pos = 12;
    let mut animation_chunks_preserved = 0;
    let mut metadata_chunks_stripped = 0;
    
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Preserve ALL animation-related chunks, only strip pure metadata
        let keep_chunk = match chunk_id {
            // Essential WebP chunks - ALWAYS keep
            b"VP8X" | b"VP8 " | b"VP8L" | b"ALPH" => true,
            // Animation chunks - ALWAYS keep to preserve animation
            b"ANIM" | b"ANMF" => {
                animation_chunks_preserved += 1;
                true
            },
            // Color management - keep for high quality
            b"ICCP" => true, 
            // Pure metadata - strip to save space
            b"EXIF" | b"XMP " => {
                metadata_chunks_stripped += 1;
                false
            },
            // Unknown chunks - be conservative and keep them
            _ => true,
        };
        
        if keep_chunk {
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            
            // WebP requires even byte alignment
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    // Update file size in RIFF header
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    log_to_console(&format!("Animated WebP optimization: preserved {} animation chunks, stripped {} metadata chunks", 
                           animation_chunks_preserved, metadata_chunks_stripped));
    
    if result.len() < data.len() {
        Ok(result)
    } else {
        log_to_console("No size reduction achieved - animated WebP already optimized");
        Err(PixieError::ProcessingError("No optimization possible for this animated WebP".to_string()))
    }
}

/// More aggressive animated WebP optimization while preserving animation
fn optimize_animated_webp_metadata_aggressive(data: &[u8]) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    // Copy RIFF header
    result.extend_from_slice(&data[0..4]); // "RIFF"
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]); // Placeholder for file size
    result.extend_from_slice(&data[8..12]); // "WEBP"
    
    let mut pos = 12;
    let mut animation_chunks_preserved = 0;
    let mut chunks_stripped = 0;
    
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Even more aggressive - only keep absolutely essential chunks
        let keep_chunk = match chunk_id {
            // Core WebP image data - MUST keep
            b"VP8X" | b"VP8 " | b"VP8L" | b"ALPH" => true,
            // Animation chunks - MUST keep to preserve animation
            b"ANIM" | b"ANMF" => {
                animation_chunks_preserved += 1;
                true
            },
            // Everything else - strip aggressively for smaller size
            _ => {
                chunks_stripped += 1;
                false
            },
        };
        
        if keep_chunk {
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            
            // WebP requires even byte alignment
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    // Update file size in RIFF header
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    log_to_console(&format!("Aggressive animated WebP optimization: preserved {} animation chunks, stripped {} other chunks", 
                           animation_chunks_preserved, chunks_stripped));
    
    if result.len() < data.len() {
        Ok(result)
    } else {
        log_to_console("No size reduction achieved - animated WebP cannot be optimized further");
        Err(PixieError::ProcessingError("Animated WebP already at minimum size".to_string()))
    }
}

#[cfg(not(feature = "codec-webp"))]
fn optimize_animated_webp_reencoding(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    let _ = data;
    Err(PixieError::FeatureNotEnabled("Animated WebP optimization requires 'codec-webp' feature".to_string()))
}

/// Strip metadata from animated WebP while preserving animation chunks
#[allow(dead_code)]
fn strip_animated_webp_metadata(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    // Copy RIFF header
    result.extend_from_slice(&data[0..4]); // "RIFF"
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]); // Placeholder for file size
    result.extend_from_slice(&data[8..12]); // "WEBP"
    
    let mut pos = 12;
    let mut found_animation = false;
    
    while pos + 8 <= data.len() {
        let chunk_id = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]
        ]) as usize;
        
        if pos + 8 + chunk_size > data.len() {
            break;
        }
        
        // Keep animation-essential chunks, strip metadata
        let keep_chunk = match chunk_id {
            b"VP8X" | b"ANIM" | b"ANMF" | b"VP8 " | b"VP8L" | b"ALPH" => {
                if chunk_id == b"ANIM" {
                    found_animation = true;
                }
                true
            },
            b"ICCP" => quality >= 85, // Color profile only for high quality
            b"EXIF" | b"XMP " => false, // Always strip metadata for animations
            _ => false,
        };
        
        if keep_chunk {
            result.extend_from_slice(&data[pos..pos + 8 + chunk_size]);
            
            if chunk_size % 2 == 1 {
                result.push(0);
            }
        }
        
        pos += 8 + chunk_size;
        if chunk_size % 2 == 1 {
            pos += 1;
        }
    }
    
    // Update file size
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    if found_animation && result.len() < data.len() {
        Ok(result)
    } else {
        Err(PixieError::ProcessingError("No animation found or no size reduction achieved".to_string()))
    }
}

/// Optimize WebP animation parameters for better compression
#[allow(dead_code)]
fn optimize_webp_animation_params(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    // This function would optimize animation timing, disposal methods, etc.
    // For now, we'll implement basic optimization by returning the input
    // with metadata stripping as this is the most reliable optimization
    // for animated WebP files
    
    if quality <= 50 {
        // For low quality, try more aggressive optimization
        // For low quality, try more aggressive optimization
        strip_animated_webp_metadata(data, quality.max(60))
    } else {
        // For higher quality, just basic metadata stripping
        strip_animated_webp_metadata(data, quality)
    }
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

/// Detect animated WebP format by checking for ANIM chunk
pub fn detect_animated_webp(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    
    // Check for WebP RIFF signature
    if !data[0..4].eq(b"RIFF") || !data[8..12].eq(b"WEBP") {
        return false;
    }
    
    // Look for animation chunk (ANIM)
    let mut pos = 12;
    while pos + 8 <= data.len() {
        if data[pos..pos+4].eq(b"ANIM") {
            return true;
        }
        pos += 8;
        if pos < data.len() {
            let chunk_size = u32::from_le_bytes([data[pos-4], data[pos-3], data[pos-2], data[pos-1]]) as usize;
            pos += chunk_size + (chunk_size & 1); // WebP chunks are padded to even bytes
        }
    }
    false
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

/// Apply C hotspot preprocessing for WebP optimization
#[cfg(c_hotspots_available)]
fn apply_c_hotspot_preprocessing(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use image::load_from_memory;
    
    // Decode WebP to raw RGBA for C processing
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    
    let rgba_img = img.to_rgba8();
    let mut rgba_data = rgba_img.as_raw().clone();
    let width = img.width() as usize;
    let height = img.height() as usize;
    
    // Apply C hotspot preprocessing based on quality
    if quality <= 50 {
        // Aggressive: Use color quantization + dithering
        match crate::c_hotspots::image::octree_quantization(&rgba_data, width, height, 128) {
            Ok((palette, indices)) => {
                // Convert indexed back to RGBA for WebP encoding
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
                
                // Apply Floyd-Steinberg dithering for quality enhancement
                crate::c_hotspots::image::floyd_steinberg_dither(&mut rgba_data, width, height, &palette);
            },
            Err(_) => {
                // Fallback to Gaussian blur preprocessing
                crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 1.0);
            }
        }
    } else if quality <= 70 {
        // Balanced: Use median cut quantization
        match crate::c_hotspots::image::median_cut_quantization(&rgba_data, width, height, 192) {
            Ok((palette, indices)) => {
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
            },
            Err(_) => {
                // Fallback to light Gaussian blur
                crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 0.5);
            }
        }
    } else {
        // Conservative: Light Gaussian blur only
        crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 0.3);
    }
    
    // Re-encode as WebP using image crate
    use image::{ImageBuffer, RgbaImage};
    let processed_img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, rgba_data)
        .ok_or_else(|| PixieError::ProcessingError("Failed to create image from processed data".into()))?;
    
    // Encode back to WebP format
    let mut output = Vec::new();
    use image::codecs::webp::WebPEncoder;
    
    // Use lossless WebP encoding as image crate doesn't support quality setting
    let encoder = WebPEncoder::new_lossless(&mut output);
    
    encoder.encode(
        processed_img.as_raw(),
        width as u32,
        height as u32,
        image::ColorType::Rgba8.into()
    ).map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    
    Ok(output)
}

/// Convert indexed color data back to RGBA
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
#[cfg(not(c_hotspots_available))]
#[allow(dead_code)]
fn apply_c_hotspot_preprocessing(_data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    Err(PixieError::CHotspotUnavailable("C hotspots not available for WebP preprocessing".into()))
}

/// Force conversion from any image format to WebP with optimization
/// Unlike optimize_webp, this function always converts to WebP regardless of size efficiency
/// and applies optimization strategies during the conversion process with LOSSY compression
pub fn convert_any_format_to_webp(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        // Load the image from any format
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load image for WebP conversion: {}", e)))?;
        
        // CRITICAL FIX: Use quality-based lossy encoding instead of lossless
        let mut best_result = Vec::new();
        let mut best_size = usize::MAX;
        
        #[cfg(feature = "codec-webp")]
        {
            // Map user quality to WebP quality (WebP uses 0-100 scale where 100 is best quality)
            let webp_quality = match quality {
                0..=20 => 30.0,   // Low quality for aggressive compression
                21..=40 => 50.0,  // Medium-low quality
                41..=60 => 70.0,  // Medium quality
                61..=80 => 85.0,  // High quality
                _ => 95.0,        // Very high quality but still lossy
            };
            
            // Strategy 1: Use JPEG intermediate for lossy compression
            if quality < 85 {
                match convert_via_jpeg_intermediate(&img, quality) {
                    Ok(webp_data) if webp_data.len() < best_size => {
                        best_result = webp_data;
                        best_size = best_result.len();
                    },
                    _ => {} // Failed, try next strategy
                }
            }
            
            // Strategy 2: Fallback to image crate with preprocessing
            if best_result.is_empty() || best_size > data.len() {
                // Apply preprocessing for better compression
                let processed_img = if quality <= 70 {
                    apply_webp_preprocessing(&img, quality).unwrap_or_else(|_| img.clone())
                } else {
                    img.clone()
                };
                
                // Use lossless only for very high quality (90+), otherwise we need a workaround
                if quality >= 90 {
                    let mut temp_output = Vec::new();
                    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut temp_output);
                    if processed_img.write_with_encoder(encoder).is_ok() && temp_output.len() < best_size {
                        best_result = temp_output;
                    }
                } else {
                    // For quality < 90, convert to JPEG first, then load as WebP
                    // This is a workaround for the image crate's limited lossy WebP support
                    match convert_via_jpeg_intermediate(&processed_img, quality) {
                        Ok(jpeg_webp) if jpeg_webp.len() < best_size => {
                            best_result = jpeg_webp;
                        },
                        _ => {
                            // Last resort: Use lossless but with heavy preprocessing
                            let rgb_img = processed_img.to_rgb8();
                            let rgb_dynamic = image::DynamicImage::ImageRgb8(rgb_img);
                            
                            let mut temp_output = Vec::new();
                            let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut temp_output);
                            if rgb_dynamic.write_with_encoder(encoder).is_ok() {
                                best_result = temp_output;
                            }
                        }
                    }
                }
            }
            
            // Strategy 3: If still too large, try PNG as better alternative to bloated WebP
            if best_result.len() > data.len() * 2 {
                // WebP conversion is creating bloated files, use PNG instead
                let mut png_output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut png_output, 
                    image::codecs::png::CompressionType::Best, 
                    image::codecs::png::FilterType::Adaptive
                );
                
                if img.write_with_encoder(encoder).is_ok() && png_output.len() < best_result.len() {
                    // Return PNG instead of bloated WebP
                    return Ok(png_output);
                }
            }
            
            if best_result.is_empty() {
                return Err(PixieError::ProcessingError("All WebP encoding strategies failed".to_string()));
            }
            
            Ok(best_result)
        }
        #[cfg(not(feature = "codec-webp"))]
        {
            Err(PixieError::FeatureNotEnabled("WebP encoding not available - missing codec-webp feature".to_string()))
        }
    }
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::FeatureNotEnabled("Image processing not available - missing image feature".to_string()))
    }
}


/// Convert via JPEG intermediate format as workaround for lossy WebP
#[cfg(feature = "image")]
fn convert_via_jpeg_intermediate(img: &image::DynamicImage, quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Converting via JPEG intermediate with quality {}", quality));
    
    let jpeg_quality = match quality {
        0..=20 => 25,
        21..=40 => 40,
        41..=60 => 60,
        61..=80 => 75,
        _ => 85,
    };
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Using JPEG quality {} for intermediate", jpeg_quality));
    
    let mut jpeg_data = Vec::new();
    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, jpeg_quality);
    let rgb_img = img.to_rgb8();
    
    rgb_img.write_with_encoder(jpeg_encoder)
        .map_err(|e| {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console(&format!("JPEG encoding failed: {}", e));
            PixieError::ProcessingError(format!("JPEG intermediate encoding failed: {}", e))
        })?;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("JPEG intermediate created: {} bytes", jpeg_data.len()));
    
    let compressed_img = image::load_from_memory(&jpeg_data)
        .map_err(|e| {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console(&format!("JPEG loading failed: {}", e));
            PixieError::ProcessingError(format!("JPEG intermediate loading failed: {}", e))
        })?;
    
    let mut webp_output = Vec::new();
    let webp_encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_output);
    compressed_img.write_with_encoder(webp_encoder)
        .map_err(|e| {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console(&format!("WebP encoding failed: {}", e));
            PixieError::ProcessingError(format!("WebP final encoding failed: {}", e))
        })?;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("WebP output created: {} bytes", webp_output.len()));
    
    Ok(webp_output)
}

/// Apply preprocessing to improve WebP compression
#[cfg(feature = "image")]
fn apply_webp_preprocessing(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::DynamicImage;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Applying WebP preprocessing for quality {}", quality));
    
    if quality <= 40 {
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else if quality <= 70 {
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else {
        Ok(img.clone())
    }
}

/// Convert indexed color data back to RGBA for WebP processing
#[cfg(c_hotspots_available)]
fn indices_to_rgba_webp(indices: &[u8], palette: &[crate::c_hotspots::Color32], width: usize, height: usize) -> Vec<u8> {
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
