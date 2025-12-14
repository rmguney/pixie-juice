extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};

use crate::types::{PixieResult, ImageOptConfig, PixieError, OptResult, OptError};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage};

#[cfg(all(feature = "image", target_arch = "wasm32"))]
use image::GenericImageView;

#[cfg(target_arch = "wasm32")]
use alloc::vec;

pub fn optimize_jpeg_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_jpeg_with_config(data, quality, &ImageOptConfig::default())
}

pub fn optimize_jpeg_with_config(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let original_size = data.len();
        
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(
                format!("Failed to load JPEG: {}", e)
            ))?;
        
        let strategies = get_jpeg_optimization_strategies(quality, &img, config);
        
        let mut best_result = data.to_vec();
        
        for strategy in strategies {
            if let Ok(optimized) = apply_jpeg_strategy(&img, strategy, quality, config) {
                if optimized.len() < best_result.len() {
                    best_result = optimized;
                }
            }
        }
        
        if best_result.len() >= data.len() * 90 / 100 {
            if let Ok(metadata_stripped) = optimize_jpeg_legacy(data, quality, config) {
                if metadata_stripped.len() < best_result.len() {
                    best_result = metadata_stripped;
                }
            }
        }
        
        if best_result.len() < original_size {
            Ok(best_result)
        } else {
            Ok(data.to_vec())
        }
    }
    
    #[cfg(not(feature = "image"))]
    {
        optimize_jpeg_legacy(data, quality, config)
    }
}

#[cfg(feature = "image")]
#[derive(Debug, Clone)]
enum JPEGOptimizationStrategy {
    ProgressiveReencode { jpeg_quality: u8 },
    ReencodeJPEG { jpeg_quality: u8 },
    ConvertToWebP { webp_quality: u8 },
    ConvertToPNG,
    ConvertToGrayscale { jpeg_quality: u8 },
}

#[cfg(feature = "image")]
fn get_jpeg_optimization_strategies(quality: u8, _img: &DynamicImage, config: &ImageOptConfig) -> Vec<JPEGOptimizationStrategy> {
    let mut strategies = Vec::new();
    
    let jpeg_quality = if config.lossless {
        95
    } else {
        match quality {
            0..=20 => 15,
            21..=40 => 30,
            41..=60 => 50,
            61..=80 => 70,
            _ => 85,
        }
    };
    strategies.push(JPEGOptimizationStrategy::ProgressiveReencode { jpeg_quality });
    
    strategies.push(JPEGOptimizationStrategy::ReencodeJPEG { jpeg_quality });
    
    if quality <= 70 && !config.lossless {
        let webp_quality = match quality {
            0..=30 => 50,
            31..=50 => 70,
            _ => 80,
        };
        strategies.push(JPEGOptimizationStrategy::ConvertToWebP { webp_quality });
    }
    
    if quality >= 90 || config.lossless {
        strategies.push(JPEGOptimizationStrategy::ConvertToPNG);
    }
    
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
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("Progressive JPEG re-encoding failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        JPEGOptimizationStrategy::ReencodeJPEG { jpeg_quality } => {
            #[cfg(c_hotspots_available)]
            if img.dimensions().0 * img.dimensions().1 > 100_000 && jpeg_quality <= 70 {
                if let Ok(preprocessed_img) = apply_jpeg_c_hotspot_preprocessing(img, jpeg_quality) {
                    return encode_jpeg_from_image(&preprocessed_img, jpeg_quality);
                }
            }
            
            encode_jpeg_from_image(img, jpeg_quality)
        },
        
        JPEGOptimizationStrategy::ConvertToWebP { webp_quality: _ } => {
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("JPEG to WebP conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        JPEGOptimizationStrategy::ConvertToPNG => {
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("JPEG to PNG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        JPEGOptimizationStrategy::ConvertToGrayscale { jpeg_quality } => {
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            let gray_img = img.to_luma8();
            gray_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("JPEG grayscale conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
    }
}

pub fn optimize_jpeg(data: &[u8], quality: u8, config: &ImageOptConfig) -> OptResult<Vec<u8>> {
    optimize_jpeg_with_config(data, quality, config)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

pub fn optimize_jpeg_legacy(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    if data.len() < 2 || !data.starts_with(&[0xFF, 0xD8]) {
        return Err(PixieError::InvalidFormat("Not a valid JPEG file".to_string()));
    }
    
    if config.lossless {
        optimize_jpeg_lossless(data, quality)
    } else {
        optimize_jpeg_lossy(data, quality)
    }
}

fn optimize_jpeg_lossless(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let mut result = Vec::with_capacity(data.len());
    let mut pos = 0;
    
    while pos + 1 < data.len() {
        if data[pos] == 0xFF {
            if pos + 1 >= data.len() {
                break;
            }
            
            let marker = data[pos + 1];
            
            match marker {
                0x00 | 0xFF => {
                    result.push(data[pos]);
                    pos += 1;
                },
                0xD8 => {
                    result.extend_from_slice(&data[pos..pos + 2]);
                    pos += 2;
                },
                0xD9 => {
                    result.extend_from_slice(&data[pos..pos + 2]);
                    break;
                },
                0xDA => {
                    result.extend_from_slice(&data[pos..]);
                    break;
                },
                0xE0..=0xEF => {
                    if let Some(segment_end) = get_segment_end(data, pos) {
                        let keep_segment = match marker {
                            0xE0 => true,
                            0xE1 => quality > 80,
                            0xE2..=0xEF => quality > 90,
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
            result.push(data[pos]);
            pos += 1;
        }
    }
    
    if !result.ends_with(&[0xFF, 0xD9]) {
        result.extend_from_slice(&[0xFF, 0xD9]);
    }
    
    if result.len() < data.len() {
        Ok(result)
    } else {
        Ok(data.to_vec())
    }
}

fn optimize_jpeg_lossy(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let lossless_result = optimize_jpeg_lossless(data, quality)?;
    
    if lossless_result.len() >= data.len() * 95 / 100 {
        optimize_jpeg_quality(data, quality)
    } else {
        Ok(lossless_result)
    }
}

fn optimize_jpeg_quality(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let quality_factor = match quality {
        0..=20 => 0.3,
        21..=40 => 0.5,
        41..=60 => 0.7,
        61..=80 => 0.85,
        _ => 0.95,
    };
    
    let target_size = (data.len() as f32 * quality_factor) as usize;
    
    if target_size < data.len() && target_size > 1000 {
        let mut result = Vec::with_capacity(target_size);
        let mut pos = 0;
        
        while pos + 1 < data.len() && result.len() < target_size {
            if data[pos] == 0xFF {
                if pos + 1 >= data.len() {
                    break;
                }
                
                let marker = data[pos + 1];
                
                match marker {
                    0xD8 => {
                        result.extend_from_slice(&data[pos..pos + 2]);
                        pos += 2;
                    },
                    0xD9 => {
                        result.extend_from_slice(&data[pos..pos + 2]);
                        break;
                    },
                    0xDA => {
                        let remaining_space = target_size.saturating_sub(result.len());
                        let available_data = data.len() - pos;
                        let data_to_copy = remaining_space.min(available_data);
                        
                        if data_to_copy >= 2 {
                            result.extend_from_slice(&data[pos..pos + data_to_copy]);
                            if !result.ends_with(&[0xFF, 0xD9]) {
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
                        if let Some(segment_end) = get_segment_end(data, pos) {
                            let segment_size = segment_end - pos;
                            
                            if marker == 0xE0 || result.len() + segment_size < target_size {
                                result.extend_from_slice(&data[pos..segment_end]);
                            }
                            pos = segment_end;
                        } else {
                            break;
                        }
                    },
                    0xFE => {
                        if let Some(segment_end) = get_segment_end(data, pos) {
                            pos = segment_end;
                        } else {
                            break;
                        }
                    },
                    _ => {
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
        
        if !result.ends_with(&[0xFF, 0xD9]) {
            if result.len() + 2 <= target_size {
                result.extend_from_slice(&[0xFF, 0xD9]);
            } else if result.len() >= 2 {
                let len = result.len();
                result[len - 2] = 0xFF;
                result[len - 1] = 0xD9;
            }
        }
        
        if result.len() < data.len() * 90 / 100 {
            return Ok(result);
        }
    }
    
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

#[cfg(feature = "image")]
fn encode_jpeg_from_image(img: &DynamicImage, jpeg_quality: u8) -> PixieResult<Vec<u8>> {
    let mut output = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
    
    let rgb_img = img.to_rgb8();
    rgb_img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ProcessingError(
            format!("JPEG encoding failed: {}", e)
        ))?;
    
    Ok(output)
}

#[cfg(all(feature = "image", c_hotspots_available))]
fn apply_jpeg_c_hotspot_preprocessing(img: &DynamicImage, quality: u8) -> PixieResult<DynamicImage> {
    let rgba_img = img.to_rgba8();
    let mut rgba_data = rgba_img.as_raw().clone();
    let width = img.width() as usize;
    let height = img.height() as usize;
    
    if quality <= 40 {
        match crate::c_hotspots::image::median_cut_quantization(&rgba_data, width, height, 64) {
            Ok((palette, indices)) => {
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
                crate::c_hotspots::image::floyd_steinberg_dither(&mut rgba_data, width, height, &palette);
            },
            Err(_) => {
                apply_yuv_color_space_optimization(&mut rgba_data);
            }
        }
    } else if quality <= 70 {
        match crate::c_hotspots::image::octree_quantization(&rgba_data, width, height, 128) {
            Ok((palette, indices)) => {
                rgba_data = indices_to_rgba(&indices, &palette, width, height);
            },
            Err(_) => {
                apply_yuv_color_space_optimization(&mut rgba_data);
            }
        }
    } else {
        apply_yuv_color_space_optimization(&mut rgba_data);
    }
    
    use image::{ImageBuffer, RgbaImage, DynamicImage};
    let processed_img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, rgba_data)
        .ok_or_else(|| PixieError::ProcessingError("Failed to create image from processed data".into()))?;
    
    Ok(DynamicImage::ImageRgba8(processed_img))
}

#[cfg(c_hotspots_available)]
fn apply_yuv_color_space_optimization(rgba_data: &mut [u8]) {
    crate::c_hotspots::image::rgba_yuv_roundtrip_inplace_simd(rgba_data);
}

#[cfg(c_hotspots_available)]
fn indices_to_rgba(indices: &[u8], palette: &[crate::c_hotspots::Color32], width: usize, height: usize) -> Vec<u8> {
    let _ = (width, height);
    let mut rgba_data = vec![0u8; indices.len() * 4];
    crate::c_hotspots::image::palette_indices_to_rgba_hotspot(
        indices,
        palette,
        &mut rgba_data,
        crate::c_hotspots::Color32 { r: 0, g: 0, b: 0, a: 255 },
    );
    rgba_data
}

#[cfg(any(not(feature = "image"), not(c_hotspots_available)))]
#[allow(dead_code)]
fn apply_jpeg_c_hotspot_preprocessing(_img: &DynamicImage, _quality: u8) -> PixieResult<DynamicImage> {
    Err(PixieError::CHotspotUnavailable("C hotspots not available for JPEG preprocessing".into()))
}

pub fn convert_any_format_to_jpeg(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load image for JPEG conversion: {}", e)))?;
        
        let mut best_result = Vec::new();
        let best_size = usize::MAX;
        
        let processed_img = if quality <= 70 {
            apply_jpeg_preprocessing(&img, quality).unwrap_or_else(|_| img.clone())
        } else {
            img.clone()
        };
        
        let jpeg_quality = match quality {
            0..=20 => 15,
            21..=40 => 30,
            41..=60 => 50,
            61..=80 => 70,
            _ => 85,
        };
        
        let strategies = [
            (jpeg_quality, false),
            (jpeg_quality, true),
        ];
        
        for (quality_setting, use_preprocessing) in strategies {
            let img_to_encode = if use_preprocessing && quality <= 50 {
                &processed_img
            } else {
                &img
            };
            
            let mut temp_output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut temp_output, quality_setting);
            
            let rgb_img = img_to_encode.to_rgb8();
            if rgb_img.write_with_encoder(encoder).is_ok() && temp_output.len() < best_size {
                best_result = temp_output;
            }
        }
        
        if quality <= 40 {
            let mut temp_output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut temp_output, jpeg_quality);
            
            let gray_img = processed_img.to_luma8();
            if gray_img.write_with_encoder(encoder).is_ok() && temp_output.len() < best_size {
                best_result = temp_output;
            }
        }
        
        if best_result.is_empty() {
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

#[cfg(feature = "image")]
fn apply_jpeg_preprocessing(img: &image::DynamicImage, quality: u8) -> PixieResult<image::DynamicImage> {
    use image::DynamicImage;
    
    if quality <= 40 {
        #[cfg(c_hotspots_available)]
        {
            let rgba_img = img.to_rgba8();
            let mut rgba_data = rgba_img.as_raw().clone();
            let width = img.width() as usize;
            let height = img.height() as usize;
            
            apply_yuv_color_space_optimization(&mut rgba_data);
            
            if let Some(processed_img) = image::ImageBuffer::from_raw(width as u32, height as u32, rgba_data) {
                return Ok(DynamicImage::ImageRgba8(processed_img));
            }
        }
        
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else if quality <= 70 {
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    } else {
        let rgb_img = img.to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_img))
    }
}
