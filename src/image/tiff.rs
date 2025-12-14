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

pub fn optimize_tiff_rust(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_tiff_with_config(data, quality, &ImageOptConfig::default())
}

pub fn optimize_tiff_with_config(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        #[cfg(target_arch = "wasm32")]
        {
            if data.len() < 100_000 {
                return optimize_tiff_safe_fallback(data, quality, config);
            }
        }
        
        let img = match load_from_memory(data) {
            Ok(img) => img,
            Err(e) => {
                return optimize_tiff_safe_fallback(data, quality, config);
            }
        };
        
        let strategies = get_tiff_optimization_strategies(quality, &img, config);
        
        let mut best_result = data.to_vec();
        
        for strategy in strategies {
            match apply_tiff_strategy(&img, strategy, quality, config) {
                Ok(optimized) => {
                    if optimized.len() < best_result.len() {
                        best_result = optimized;
                    }
                },
                Err(_) => {
                    continue;
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
        Err(PixieError::FeatureNotEnabled("TIFF optimization requires 'image' feature".to_string()))
    }
}

fn optimize_tiff_safe_fallback(data: &[u8], quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        match load_from_memory(data) {
            Ok(img) => {
                let mut best_result = data.to_vec();
                
                if quality >= 70 {
                    if let Ok(png_data) = convert_to_png_safe(&img) {
                        if png_data.len() < best_result.len() {
                            best_result = png_data;
                        }
                    }
                }
                
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
                Ok(data.to_vec())
            }
        }
    }
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, quality, _config);
        Ok(data.to_vec())
    }
}

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

#[cfg(feature = "image")]
fn has_transparency_check(img: &DynamicImage) -> bool {
    match img {
        DynamicImage::ImageRgba8(rgba_img) => {
            let total_pixels = rgba_img.pixels().len();
            let sample_step = (total_pixels / 100).max(1);
            
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
    LZWCompressionCHotspot,
    LZWCompression,
    JPEGCompression { jpeg_quality: u8 },
    StripMetadataCHotspot,
    StripMetadata,
    ApplyPredictorCHotspot { predictor_type: u8 },
    OptimizeColorspaceCHotspot { target_bits: u8 },
    ConvertToPNG,
    ConvertToJPEG { jpeg_quality: u8 },
    ConvertToWebP { webp_quality: u8 },
}

#[cfg(feature = "image")]
fn get_tiff_optimization_strategies(quality: u8, img: &DynamicImage, config: &ImageOptConfig) -> Vec<TIFFOptimizationStrategy> {
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
    
    if config.lossless || quality >= 80 {
        strategies.push(TIFFOptimizationStrategy::LZWCompression);
    }
    
    strategies.push(TIFFOptimizationStrategy::StripMetadata);
    
    if !has_transparency && quality <= 75 && !config.lossless {
        let jpeg_quality = match quality {
            0..=30 => 40,
            31..=50 => 60,
            51..=70 => 75,
            _ => 85,
        };
        strategies.push(TIFFOptimizationStrategy::JPEGCompression { jpeg_quality });
    }
    
    if quality >= 70 || config.lossless {
        strategies.push(TIFFOptimizationStrategy::ConvertToPNG);
    }
    
    if !has_transparency && quality <= 85 && !config.lossless {
        let jpeg_quality = match quality {
            0..=30 => 50,
            31..=50 => 70,
            51..=70 => 80,
            _ => 90,
        };
        strategies.push(TIFFOptimizationStrategy::ConvertToJPEG { jpeg_quality });
    }
    
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
            #[cfg(target_arch = "wasm32")]
            {
                apply_tiff_strategy(img, TIFFOptimizationStrategy::LZWCompression, quality, _config)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let rgba_img = img.to_rgba8();
                let (width, height) = (rgba_img.width() as usize, rgba_img.height() as usize);
                
                compress_tiff_lzw_c_hotspot(rgba_img.as_raw(), width, height, quality)
            }
        },
        
        TIFFOptimizationStrategy::StripMetadataCHotspot => {
            #[cfg(target_arch = "wasm32")]
            {
                apply_tiff_strategy(img, TIFFOptimizationStrategy::StripMetadata, quality, _config)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let mut output = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new(&mut output);
                
                img.write_with_encoder(encoder)
                    .map_err(|e| PixieError::ProcessingError(
                        format!("TIFF metadata stripping failed: {}", e)
                    ))?;
                
                strip_tiff_metadata_c_hotspot(&output, false)
            }
        },
        
        TIFFOptimizationStrategy::ApplyPredictorCHotspot { predictor_type } => {
            #[cfg(target_arch = "wasm32")]
            {
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
                let mut rgba_img = img.to_rgba8();
                let (width, height) = (rgba_img.width() as usize, rgba_img.height() as usize);
                
                apply_tiff_predictor_c_hotspot(rgba_img.as_mut(), width, height, predictor_type)?;
                
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
            #[cfg(target_arch = "wasm32")]
            {
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
                let mut rgba_img = img.to_rgba8();
                let (width, height) = (rgba_img.width() as usize, rgba_img.height() as usize);
                
                optimize_tiff_colorspace_c_hotspot(rgba_img.as_mut(), width, height, target_bits)?;
                
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
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF metadata stripping failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        TIFFOptimizationStrategy::ConvertToPNG => {
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            
            img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF to PNG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        TIFFOptimizationStrategy::ConvertToJPEG { jpeg_quality } => {
            let mut output = Vec::new();
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, jpeg_quality);
            
            let rgb_img = img.to_rgb8();
            rgb_img.write_with_encoder(encoder)
                .map_err(|e| PixieError::ProcessingError(
                    format!("TIFF to JPEG conversion failed: {}", e)
                ))?;
            
            Ok(output)
        },
        
        TIFFOptimizationStrategy::ConvertToWebP { webp_quality: _ } => {
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

pub fn is_tiff(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    
    (data[0..2] == [0x49, 0x49] && data[2] == 42 && data[3] == 0) ||
    (data[0..2] == [0x4D, 0x4D] && data[2] == 0 && data[3] == 42)
}

pub fn optimize_tiff(data: &[u8], quality: u8) -> OptResult<Vec<u8>> {
    optimize_tiff_rust(data, quality)
        .map_err(|e| OptError::ProcessingError(e.to_string()))
}
