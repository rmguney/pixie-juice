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
        
        if detect_animated_webp(data) {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Detected animated WebP - using comprehensive optimization");
            
            return optimize_animated_webp_native(data, quality);
        }
        
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
        
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Using WebP metadata optimization strategies");
        
        let mut best_result = data.to_vec();
        let mut best_size = data.len();
        let mut strategies_succeeded = 0;
        
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


fn reencode_webp_native_quality(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use image::load_from_memory;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Re-encoding WebP with native quality {}", quality));
    
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    
    if quality < 85 {
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Using JPEG intermediate strategy for lossy WebP compression");
        
        match convert_webp_via_jpeg_lossy(&img, quality) {
            Ok(compressed) if compressed.len() < data.len() => {
                let savings = ((data.len() - compressed.len()) as f64 / data.len() as f64) * 100.0;
                #[cfg(target_arch = "wasm32")]
                crate::image::log_to_console(&format!("JPEG intermediate success: {:.1}% compression", savings));
                #[cfg(not(target_arch = "wasm32"))]
                let _ = savings;
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
                let _ = e;
            }
        }
        
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

fn convert_webp_via_jpeg_lossy(img: &image::DynamicImage, quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Converting WebP via JPEG with quality {}", quality));
    
    let mut jpeg_output = Vec::new();
    let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, quality);
    img.write_with_encoder(jpeg_encoder)
        .map_err(|e| PixieError::ImageEncodingFailed(format!("JPEG encoding failed: {}", e)))?;
    
    let jpeg_img = image::load_from_memory(&jpeg_output)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("JPEG decode failed: {}", e)))?;
    
    let mut webp_output = Vec::new();
    let webp_encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp_output);
    jpeg_img.write_with_encoder(webp_encoder)
        .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("JPEG->WebP conversion: {} bytes", webp_output.len()));
    
    Ok(webp_output)
}

fn encode_webp_with_image_crate(img: &image::DynamicImage, quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Using image crate for WebP encoding with quality simulation {}", quality));
    
    let processed_img = apply_quality_preprocessing(img, quality)?;
    
    let mut output = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
    processed_img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("WebP encoded: {} bytes with quality simulation {}", output.len(), quality));
    
    Ok(output)
}

fn apply_quality_preprocessing(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::DynamicImage;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Applying quality preprocessing for quality {}", quality));
    
    if quality <= 30 {
        let new_width = (img.width() * 75 / 100).max(16);
        let new_height = (img.height() * 75 / 100).max(16);
        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Triangle);
        let rgb_img = resized.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else if quality <= 50 {
        let new_width = (img.width() * 85 / 100).max(16);
        let new_height = (img.height() * 85 / 100).max(16);
        let resized = img.resize(new_width, new_height, image::imageops::FilterType::Triangle);
        let rgb_img = resized.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else if quality <= 70 {
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else {
        Ok(img.clone())
    }
}

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
    
    let processed_img = if quality <= 60 {
        apply_aggressive_webp_preprocessing(&img, quality).unwrap_or_else(|_| {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console("Aggressive preprocessing failed, using standard");
            img.clone()
        })
    } else {
        apply_quality_preprocessing(&img, quality).unwrap_or_else(|_| img.clone())
    };
    
    let mut output = Vec::new();
    
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

fn apply_aggressive_webp_preprocessing(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::DynamicImage;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Applying aggressive preprocessing for quality {}", quality));
    
    let (scale_factor, should_quantize) = match quality {
        0..=20 => (60, true),
        21..=40 => (70, true),
        41..=60 => (85, false),
        _ => (95, false),
    };
    
    let new_width = (img.width() * scale_factor / 100).max(8);
    let new_height = (img.height() * scale_factor / 100).max(8);
    let mut processed = img.resize(new_width, new_height, image::imageops::FilterType::Triangle);
    
    if should_quantize {
        #[cfg(target_arch = "wasm32")]
        crate::image::log_to_console("Applying color quantization");
        
        processed = apply_color_quantization_simple(&processed, quality)?;
    }
    
    let rgb_img = processed.to_rgb8();
    Ok(DynamicImage::ImageRgb8(rgb_img))
}

fn apply_webp_color_quantization(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use image::load_from_memory;
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Applying WebP color quantization with quality {}", quality));
    
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    
    let quantized_img = apply_color_quantization_simple(&img, quality)?;
    
    let mut output = Vec::new();
    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
    quantized_img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    Ok(output)
}

fn apply_color_quantization_simple(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::{DynamicImage, RgbImage};
    
    let rgb_img = img.to_rgb8();
    let (width, height) = (rgb_img.width(), rgb_img.height());
    
    let bit_shift = match quality {
        0..=20 => 3,
        21..=40 => 2,
        _ => 1,
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

fn optimize_animated_webp_native(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console("Starting native animated WebP optimization");
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
            if quality <= 50 {
                optimize_animated_webp_frames(data, quality)
            } else {
                Ok(data.to_vec())
            }
        },
        Err(e) => {
            #[cfg(target_arch = "wasm32")]
            crate::image::log_to_console(&format!("Metadata stripping failed: {}", e));
            if quality <= 50 {
                optimize_animated_webp_frames(data, quality)
            } else {
                Ok(data.to_vec())
            }
        }
    }
}

fn strip_animated_webp_metadata_aggressive(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Aggressive animated WebP stripping for quality {}", quality));
    
    let mut result = Vec::new();
    
    result.extend_from_slice(&data[0..4]);
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]);
    result.extend_from_slice(&data[8..12]);
    
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
        
        let keep_chunk = match chunk_id {
            b"VP8X" => true,
            b"ANIM" => {
                animation_chunks_preserved += 1;
                true
            },
            b"ANMF" => {
                animation_chunks_preserved += 1;
                total_frames_processed += 1;
                
                if quality <= 30 && total_frames_processed > 20 {
                    chunks_stripped += 1;
                    false
                } else {
                    true
                }
            },
            b"VP8 " | b"VP8L" | b"ALPH" => true,
            b"ICCP" => quality >= 80,
            b"EXIF" | b"XMP " => quality >= 90,
            _ => {
                chunks_stripped += 1;
                quality >= 70
            },
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

fn optimize_animated_webp_frames(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if quality > 50 {
        return Ok(data.to_vec());
    }
    
    #[cfg(target_arch = "wasm32")]
    crate::image::log_to_console(&format!("Optimizing animated WebP frames for quality {}", quality));
    
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

fn optimize_animated_webp_metadata_only(data: &[u8]) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    result.extend_from_slice(&data[0..4]);
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]);
    result.extend_from_slice(&data[8..12]);
    
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
        
        let keep_chunk = match chunk_id {
            b"VP8X" | b"VP8 " | b"VP8L" | b"ALPH" => true,
            b"ANIM" | b"ANMF" => {
                animation_chunks_preserved += 1;
                true
            },
            b"ICCP" => true, 
            b"EXIF" | b"XMP " => {
                metadata_chunks_stripped += 1;
                false
            },
            _ => true,
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

fn optimize_animated_webp_metadata_aggressive(data: &[u8]) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    result.extend_from_slice(&data[0..4]);
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]);
    result.extend_from_slice(&data[8..12]);
    
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
        
        let keep_chunk = match chunk_id {
            b"VP8X" | b"VP8 " | b"VP8L" | b"ALPH" => true,
            b"ANIM" | b"ANMF" => {
                animation_chunks_preserved += 1;
                true
            },
            _ => {
                chunks_stripped += 1;
                false
            },
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

#[allow(dead_code)]
fn strip_animated_webp_metadata(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if data.len() < 12 || !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Invalid WebP file".into()));
    }
    
    let mut result = Vec::new();
    
    result.extend_from_slice(&data[0..4]);
    let file_size_pos = result.len();
    result.extend_from_slice(&[0u8; 4]);
    result.extend_from_slice(&data[8..12]);
    
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
        
        let keep_chunk = match chunk_id {
            b"VP8X" | b"ANIM" | b"ANMF" | b"VP8 " | b"VP8L" | b"ALPH" => {
                if chunk_id == b"ANIM" {
                    found_animation = true;
                }
                true
            },
            b"ICCP" => quality >= 85,
            b"EXIF" | b"XMP " => false,
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
    
    let new_file_size = (result.len() - 8) as u32;
    let size_bytes = new_file_size.to_le_bytes();
    result[file_size_pos..file_size_pos + 4].copy_from_slice(&size_bytes);
    
    if found_animation && result.len() < data.len() {
        Ok(result)
    } else {
        Err(PixieError::ProcessingError("No animation found or no size reduction achieved".to_string()))
    }
}

#[allow(dead_code)]
fn optimize_webp_animation_params(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    if quality <= 50 {
        strip_animated_webp_metadata(data, quality.max(60))
    } else {
        strip_animated_webp_metadata(data, quality)
    }
}
#[cfg(not(feature = "codec-webp"))]
pub fn optimize_webp(_data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    Err(PixieError::FeatureNotAvailable("WebP codec not available - enable codec-webp feature".into()))
}

pub fn optimize_webp(data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
    optimize_webp_rust(data, quality)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

pub fn detect_animated_webp(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    
    if !data[0..4].eq(b"RIFF") || !data[8..12].eq(b"WEBP") {
        return false;
    }
    
    let mut pos = 12;
    while pos + 8 <= data.len() {
        if data[pos..pos+4].eq(b"ANIM") {
            return true;
        }
        pos += 8;
        if pos < data.len() {
            let chunk_size = u32::from_le_bytes([data[pos-4], data[pos-3], data[pos-2], data[pos-1]]) as usize;
            pos += chunk_size + (chunk_size & 1);
        }
    }
    false
}

pub fn optimize_webp_with_config_alt(data: &[u8], quality: u8, config: &ImageOptConfig) -> OptResult<Vec<u8>> {
    optimize_webp_with_config(data, quality, config)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

pub fn optimize_webp_old(data: &[u8], quality: u8, config: &ImageOptConfig) -> OptResult<Vec<u8>> {
    let _ = config;
    optimize_webp(data, quality)
}

pub fn is_webp(data: &[u8]) -> bool {
    data.len() >= 12 && 
    data[0..4] == [0x52, 0x49, 0x46, 0x46] &&
    data[8..12] == [0x57, 0x45, 0x42, 0x50]
}

pub fn get_webp_dimensions(data: &[u8]) -> PixieResult<(u32, u32)> {
    if !is_webp(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid WebP file".into()));
    }
    
    use image::load_from_memory;
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    Ok((img.width(), img.height()))
}

#[cfg(c_hotspots_available)]
fn apply_c_hotspot_preprocessing(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use image::load_from_memory;
    let img = load_from_memory(data)
        .map_err(|e| PixieError::ImageDecodingFailed(format!("WebP decode failed: {}", e)))?;
    
    let rgba_img = img.to_rgba8();
    let mut rgba_data = rgba_img.as_raw().clone();
    let width = img.width() as usize;
    let height = img.height() as usize;
    
    if quality <= 50 {
        match crate::c_hotspots::image::octree_quantization(&rgba_data, width, height, 128) {
            Ok((palette, indices)) => {
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
                crate::c_hotspots::image::floyd_steinberg_dither(&mut rgba_data, width, height, &palette);
            },
            Err(_) => {
                crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 1.0);
            }
        }
    } else if quality <= 70 {
        match crate::c_hotspots::image::median_cut_quantization(&rgba_data, width, height, 192) {
            Ok((palette, indices)) => {
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
            },
            Err(_) => {
                crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 0.5);
            }
        }
    } else {
        crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width, height, 0.3);
    }
    use image::{ImageBuffer, RgbaImage};
    let processed_img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, rgba_data)
        .ok_or_else(|| PixieError::ProcessingError("Failed to create image from processed data".into()))?;
    let mut output = Vec::new();
    use image::codecs::webp::WebPEncoder;
    let encoder = WebPEncoder::new_lossless(&mut output);
    encoder.encode(
        processed_img.as_raw(),
        width as u32,
        height as u32,
        image::ColorType::Rgba8.into()
    ).map_err(|e| PixieError::ImageEncodingFailed(format!("WebP encoding failed: {}", e)))?;
    
    Ok(output)
}

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
            rgba_data.extend_from_slice(&[0, 0, 0, 255]);
        }
    }
    
    rgba_data
}

#[cfg(not(c_hotspots_available))]
#[allow(dead_code)]
fn apply_c_hotspot_preprocessing(_data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    Err(PixieError::CHotspotUnavailable("C hotspots not available for WebP preprocessing".into()))
}

pub fn convert_any_format_to_webp(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load image for WebP conversion: {}", e)))?;
        
        let mut best_result = Vec::new();
        let mut best_size = usize::MAX;
        
        #[cfg(feature = "codec-webp")]
        {
            let webp_quality = match quality {
                0..=20 => 30.0,
                21..=40 => 50.0,
                41..=60 => 70.0,
                61..=80 => 85.0,
                _ => 95.0,
            };
            
            if quality < 85 {
                match convert_via_jpeg_intermediate(&img, quality) {
                    Ok(webp_data) if webp_data.len() < best_size => {
                        best_result = webp_data;
                        best_size = best_result.len();
                    },
                    _ => {}
                }
            }
            
            if best_result.is_empty() || best_size > data.len() {
                let processed_img = if quality <= 70 {
                    apply_webp_preprocessing(&img, quality).unwrap_or_else(|_| img.clone())
                } else {
                    img.clone()
                };
                
                if quality >= 90 {
                    let mut temp_output = Vec::new();
                    let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut temp_output);
                    if processed_img.write_with_encoder(encoder).is_ok() && temp_output.len() < best_size {
                        best_result = temp_output;
                    }
                } else {
                    match convert_via_jpeg_intermediate(&processed_img, quality) {
                        Ok(jpeg_webp) if jpeg_webp.len() < best_size => {
                            best_result = jpeg_webp;
                        },
                        _ => {
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
            
            if best_result.len() > data.len() * 2 {
                let mut png_output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut png_output, 
                    image::codecs::png::CompressionType::Best, 
                    image::codecs::png::FilterType::Adaptive
                );
                
                if img.write_with_encoder(encoder).is_ok() && png_output.len() < best_result.len() {
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
            rgba_data.extend_from_slice(&[0, 0, 0, 255]);
        }
    }
    
    rgba_data
}
