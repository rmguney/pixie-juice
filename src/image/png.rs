extern crate alloc;
use alloc::{vec, vec::Vec, string::ToString, format};

use crate::types::{OptResult, OptError, PixieResult, ImageOptConfig};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage};

#[cfg(all(feature = "image", target_arch = "wasm32"))]
use image::GenericImageView;
use image::codecs::png::{PngEncoder, CompressionType, FilterType};

pub fn optimize_png_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_png_with_config(data, quality, &ImageOptConfig::default())
}

pub fn optimize_png_with_config(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let img = load_from_memory(data)
            .map_err(|e| crate::types::PixieError::ProcessingError(
                format!("Failed to load PNG: {}", e)
            ))?;
        
        let strategies = get_png_optimization_strategies(quality, &img);
        
        let mut best_result = data.to_vec();
        
        for strategy in strategies {
            if let Ok(optimized) = apply_png_strategy(&img, strategy, quality, config, data.len()) {
                if optimized.len() < best_result.len() {
                    best_result = optimized;
                }
            }
        }
        
        if data.len() < 5_000_000 {
            if let Ok(quantized) = apply_aggressive_color_quantization(&img, quality) {
                if quantized.len() < best_result.len() {
                    best_result = quantized;
                }
            }
        }
        
        if best_result.len() < data.len() {
            Ok(best_result)
        } else {
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
    AggressiveReencode { compression_level: u8 },
    ReencodePNG { compression_level: u8 },
    ConvertToJPEG { jpeg_quality: u8 },
    ConvertToWebP { webp_quality: u8 },
    PaletteOptimization,
}

#[cfg(feature = "image")]
fn get_png_optimization_strategies(quality: u8, img: &DynamicImage) -> Vec<PNGOptimizationStrategy> {
    let mut strategies = Vec::new();
    
    let has_transparency = match img {
        DynamicImage::ImageRgba8(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < 255)
        },
        DynamicImage::ImageRgba16(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < u16::MAX)
        },
        _ => false,
    };
    
    let compression_level = match quality {
        0..=30 => 9,
        31..=60 => 9,
        61..=80 => 9,
        _ => 9,
    };
    
    strategies.push(PNGOptimizationStrategy::AggressiveReencode { compression_level });
    
    if !has_transparency {
        let jpeg_quality = match quality {
            0..=30 => 15,
            31..=60 => 25,
            61..=80 => 35,
            _ => 45,
        };
        strategies.push(PNGOptimizationStrategy::ConvertToJPEG { jpeg_quality });
    }
    
    if quality <= 85 {
        let webp_quality = match quality {
            0..=30 => 50,
            31..=60 => 70,
            61..=80 => 85,
            _ => 90,
        };
        strategies.push(PNGOptimizationStrategy::ConvertToWebP { webp_quality });
    }
    
    let (width, height) = (img.width(), img.height());
    let pixel_count = (width * height) as usize;
    
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
            let mut best_output = Vec::new();
            let best_size = usize::MAX;
            
            let final_img = img;
            
            let compression_type = match compression_level {
                1..=6 => CompressionType::Default,
                7..=8 => CompressionType::Best,
                9 => CompressionType::Best,
                _ => CompressionType::Best,
            };
            
            let filter_types = [
                FilterType::Adaptive,
                FilterType::NoFilter,
                FilterType::Sub,
                FilterType::Up,
                FilterType::Avg,
                FilterType::Paeth,
            ];
            
            for filter_type in filter_types {
                if let DynamicImage::ImageRgba8(rgba_img) = final_img {
                    let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
                    
                    if !has_transparency {
                        let mut rgb_output = Vec::new();
                        let rgb_encoder = PngEncoder::new_with_quality(&mut rgb_output, compression_type, filter_type);
                        let rgb_img = final_img.to_rgb8();
                        
                        if rgb_img.write_with_encoder(rgb_encoder).is_ok() && rgb_output.len() < best_size {
                            best_output = rgb_output;
                        }
                    }
                }
                
                let mut compressed_output = Vec::new();
                let encoder = PngEncoder::new_with_quality(&mut compressed_output, compression_type, filter_type);
                
                if img.write_with_encoder(encoder).is_ok() && compressed_output.len() < best_size {
                    best_output = compressed_output;
                }
                
                let has_any_transparency = match img {
                    DynamicImage::ImageRgba8(rgba_img) => rgba_img.pixels().any(|p| p[3] < 255),
                    DynamicImage::ImageRgba16(rgba_img) => rgba_img.pixels().any(|p| p[3] < u16::MAX),
                    _ => false,
                };
                
                if !has_any_transparency {
                    let mut gray_output = Vec::new();
                    let gray_encoder = PngEncoder::new_with_quality(&mut gray_output, compression_type, filter_type);
                    let gray_img = img.to_luma8();
                    
                    if gray_img.write_with_encoder(gray_encoder).is_ok() && gray_output.len() < best_size {
                        best_output = gray_output;
                    }
                }
            }
            
            if best_output.is_empty() {
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
            let mut best_output = Vec::new();
            let best_size = usize::MAX;
            
            let compression_type = match compression_level {
                1..=3 => CompressionType::Fast,
                4..=6 => CompressionType::Default,
                7..=9 => CompressionType::Best,
                _ => CompressionType::Default,
            };
            
            let mut compressed_output = Vec::new();
            let encoder = PngEncoder::new_with_quality(&mut compressed_output, compression_type, FilterType::Adaptive);
            
            if img.write_with_encoder(encoder).is_ok() && !compressed_output.is_empty() {
                if compressed_output.len() < best_size {
                    best_output = compressed_output;
                }
            }
            
            if let DynamicImage::ImageRgba8(rgba_img) = img {
                let has_transparency = rgba_img.pixels().any(|p| p[3] < 255);
                
                if !has_transparency {
                    let mut rgb_output = Vec::new();
                    let rgb_encoder = PngEncoder::new_with_quality(&mut rgb_output, compression_type, FilterType::Adaptive);
                    let rgb_img = img.to_rgb8();
                    
                    if rgb_img.write_with_encoder(rgb_encoder).is_ok() && rgb_output.len() < best_size {
                        best_output = rgb_output;
                    }
                }
            }
            
            let has_any_transparency = match img {
                DynamicImage::ImageRgba8(rgba_img) => rgba_img.pixels().any(|p| p[3] < 255),
                DynamicImage::ImageRgba16(rgba_img) => rgba_img.pixels().any(|p| p[3] < u16::MAX),
                _ => false,
            };
            
            if !has_any_transparency {
                let mut gray_output = Vec::new();
                let gray_encoder = PngEncoder::new_with_quality(&mut gray_output, compression_type, FilterType::Adaptive);
                let gray_img = img.to_luma8();
                
                if gray_img.write_with_encoder(gray_encoder).is_ok() && gray_output.len() < best_size {
                    best_output = gray_output;
                }
            }
            
            if best_output.is_empty() {
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
            let ultra_aggressive_quality = match jpeg_quality {
                0..=30 => 10,
                31..=60 => 15,
                61..=80 => 20,
                _ => 25,
            };
            
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, ultra_aggressive_quality);
            
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| crate::types::PixieError::ProcessingError(
                    format!("PNG to JPEG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        PNGOptimizationStrategy::ConvertToWebP { webp_quality: _ } => {
            #[cfg(feature = "codec-webp")]
            {
                let mut output = Vec::new();
                let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
                
                img.write_with_encoder(encoder)
                    .map_err(|e| crate::types::PixieError::ProcessingError(
                        format!("PNG to WebP conversion failed: {}", e)
                    ))?;
                
                Ok(output)
            }
            #[cfg(not(feature = "codec-webp"))]
            {
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
            #[cfg(feature = "color_quant")]
            {
                use color_quant::NeuQuant;
                
                let rgba_img = img.to_rgba8();
                let rgba_data = rgba_img.as_raw();
                
                let nq = NeuQuant::new(10, 128, rgba_data);
                let _palette = nq.color_map_rgba();
                
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

pub fn optimize_png(data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
    optimize_png_rust(data, quality)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}

#[cfg(feature = "image")]
fn apply_aggressive_color_quantization(img: &DynamicImage, quality: u8) -> PixieResult<Vec<u8>> {
    let has_transparency = match img {
        DynamicImage::ImageRgba8(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < 255)
        },
        DynamicImage::ImageRgba16(rgba_img) => {
            rgba_img.pixels().any(|p| p[3] < u16::MAX)
        },
        _ => false,
    };
    
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

#[cfg(all(feature = "image", c_hotspots_available))]
fn apply_simd_preprocessing(img: &DynamicImage) -> Option<DynamicImage> {
    let (width, height) = img.dimensions();
    if width * height < 100_000 {
        return None;
    }
    
    let mut rgba_img = img.to_rgba8();
    let mut rgba_data = rgba_img.as_raw().clone();
    
    match crate::c_hotspots::image::octree_quantization(&rgba_data, width as usize, height as usize, 256) {
        Ok((palette, indices)) => {
            rgba_data = indices_to_rgba_png(&indices, &palette, width as usize, height as usize);
            
            crate::c_hotspots::image::floyd_steinberg_dither(&mut rgba_data, width as usize, height as usize, &palette);
        },
        Err(_) => {
            crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width as usize, height as usize, 0.5);
        }
    }
    
    crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width as usize, height as usize, 0.3);
    
    use image::{ImageBuffer, RgbaImage};
    if let Some(processed_img) = ImageBuffer::from_raw(width, height, rgba_data) {
        Some(DynamicImage::ImageRgba8(processed_img))
    } else {
        None
    }
}

#[cfg(c_hotspots_available)]
fn indices_to_rgba_png(indices: &[u8], palette: &[crate::c_hotspots::Color32], width: usize, height: usize) -> Vec<u8> {
    let _ = (width, height);
    let mut rgba_data = vec![0u8; indices.len() * 4];
    crate::c_hotspots::image::palette_indices_to_rgba_hotspot(
        indices,
        palette,
        &mut rgba_data,
        crate::c_hotspots::Color32 { r: 0, g: 0, b: 0, a: 0 },
    );
    rgba_data
}

#[cfg(any(not(feature = "image"), not(c_hotspots_available)))]
fn apply_simd_preprocessing(_img: &DynamicImage) -> Option<DynamicImage> {
    None
}

pub fn convert_any_format_to_png(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        let img = load_from_memory(data)
            .map_err(|e| crate::types::PixieError::ProcessingError(format!("Failed to load image for PNG conversion: {}", e)))?;
        
        let mut temp_output = Vec::new();
        let encoder = PngEncoder::new_with_quality(&mut temp_output, CompressionType::Best, FilterType::Adaptive);
        
        img.write_with_encoder(encoder)
            .map_err(|e| crate::types::PixieError::ProcessingError(format!("PNG encoding failed: {}", e)))?;
        
        optimize_png_rust(&temp_output, 85)
    }
    #[cfg(not(feature = "image"))]
    {
        Err(crate::types::PixieError::FeatureNotEnabled("Image processing not available - missing image feature".to_string()))
    }
}


