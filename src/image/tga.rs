extern crate alloc;
use alloc::{vec::Vec, string::ToString, format};
extern crate std;
use std::io;

use crate::types::{PixieResult, ImageOptConfig, PixieError};
use crate::optimizers::{get_current_time_ms, update_performance_stats};

#[cfg(feature = "image")]
use image::{load_from_memory, DynamicImage};

pub fn optimize_tga_entry(data: &[u8], quality: u8) -> Result<Vec<u8>, wasm_bindgen::JsValue> {
    let start_time = get_current_time_ms();
    let _data_size = data.len();

    let result = optimize_tga(data, quality);
    
    match result {
        Ok(optimized_data) => {
            let elapsed_ms = get_current_time_ms() - start_time;
            update_performance_stats(true, elapsed_ms, optimized_data.len());
            Ok(optimized_data)
        }
        Err(e) => {
            Err(wasm_bindgen::JsValue::from_str(&format!("TGA optimization failed: {}", e)))
        }
    }
}

pub fn optimize_tga(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let config = ImageOptConfig::with_quality(quality);

    #[cfg(feature = "image")]
    {
        let img_result = image::load_from_memory_with_format(data, image::ImageFormat::Tga);
        if let Ok(img) = img_result {
            return optimize_tga_with_image_crate(&img, &config);
        }
        
        if let Ok(img) = image::load_from_memory(data) {
            return optimize_tga_with_image_crate(&img, &config);
        }
    }

    #[cfg(feature = "codec-tga")]
    {
        if let Ok(optimized) = optimize_tga_with_basic_processing(data, &config) {
            return Ok(optimized);
        }
    }

    if is_tga(data) {
        return convert_tga_to_png(data);
    }

    Err(PixieError::ProcessingError("Unable to process TGA file".to_string()))
}

pub fn optimize_tga_with_quality(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    optimize_tga(data, quality)
}

#[cfg(feature = "image")]
fn optimize_tga_with_image_crate(img: &DynamicImage, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    let mut buffer = Vec::new();
    
    if config.lossless {
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ProcessingError(format!("PNG encoding failed: {}", e)))?;
    } else {
        let rgb_img = img.to_rgb8();
        let quality = config.quality.min(100);
        match image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality) {
            encoder => {
                if rgb_img.write_with_encoder(encoder).is_err() {
                    buffer.clear();
                    let png_encoder = image::codecs::png::PngEncoder::new(&mut buffer);
                    img.write_with_encoder(png_encoder)
                        .map_err(|e| PixieError::ProcessingError(format!("PNG fallback failed: {}", e)))?;
                }
            }
        }
    }
    
    Ok(buffer)
}

fn convert_tga_to_png(data: &[u8]) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to decode TGA: {}", e)))?;
        
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to encode PNG: {}", e)))?;
        
        Ok(buffer)
    }
    
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::ProcessingError("Image crate not available".to_string()))
    }
}

pub fn convert_any_format_to_tga(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to decode image: {}", e)))?;
        
        let mut buffer = Vec::new();
        let encoder = image::codecs::tga::TgaEncoder::new(&mut buffer);
        img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to encode TGA: {}", e)))?;
        
        Ok(buffer)
    }
    
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::ProcessingError("Image crate not available for TGA conversion".to_string()))
    }
}

pub fn is_tga(data: &[u8]) -> bool {
    if data.len() < 18 {
        return false;
    }

    let id_length = data[0];
    let color_map_type = data[1];
    let image_type = data[2];
    
    match image_type {
        0 | 1 | 2 | 3 | 9 | 10 | 11 => {},
        _ => return false,
    }
    
    if color_map_type > 1 {
        return false;
    }
    
    let width = u16::from_le_bytes([data[12], data[13]]) as u32;
    let height = u16::from_le_bytes([data[14], data[15]]) as u32;
    let bits_per_pixel = data[16];
    
    if width == 0 || height == 0 || width > 65535 || height > 65535 {
        return false;
    }
    
    match bits_per_pixel {
        8 | 15 | 16 | 24 | 32 => {},
        _ => return false,
    }
    
    let header_size = 18 + id_length as usize;
    let expected_min_size = header_size + ((width * height * bits_per_pixel as u32) / 8) as usize;
    
    data.len() >= expected_min_size.min(header_size + 1000)
}

#[cfg(feature = "codec-tga")]
fn optimize_tga_with_basic_processing(data: &[u8], config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    let bit_depth = if data.len() >= 17 { data[16] } else { 24 };
    
    match bit_depth {
        16 => {
            optimize_16bit_tga_in_place(data, config)
        },
        32 => {
            convert_tga_to_png_simple(data, config)
        },
        _ => {
            optimize_16bit_tga_in_place(data, config)
        }
    }
}

#[cfg(feature = "codec-tga")]
fn convert_tga_to_png_simple(data: &[u8], _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load TGA: {}", e)))?;
        
        let mut buffer = Vec::new();
        img.write_to(&mut io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to write PNG: {}", e)))?;
        
        Ok(buffer)
    }
    #[cfg(not(feature = "image"))]
    {
        Ok(data.to_vec())
    }
}

#[cfg(feature = "codec-tga")]
fn optimize_16bit_tga_in_place(data: &[u8], config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    let mut result = data.to_vec();
    
    if result.len() > 26 {
        let footer_start = result.len() - 26;
        if result.len() >= footer_start + 16 && &result[footer_start..footer_start + 16] == b"TRUEVISION-XFILE" {
            result.truncate(footer_start);
        }
    }
    
    if !config.preserve_metadata && result.len() >= 18 {
        result[8..12].copy_from_slice(&[0, 0, 0, 0]);
        result[4..8].copy_from_slice(&[0, 0, 0, 0]);
    }
    
    Ok(result)
}

#[cfg(feature = "image")]
fn encode_rgb_to_png(rgb_data: &[u8], width: u32, height: u32) -> PixieResult<Vec<u8>> {
    use image::{ImageBuffer, Rgb};
    
    let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb_data)
        .ok_or_else(|| PixieError::ProcessingError("Failed to create image buffer".to_string()))?;
    
    let mut buffer = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
    img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ProcessingError(format!("PNG encoding failed: {}", e)))?;
    
    Ok(buffer)
}

#[cfg(feature = "image")]
fn encode_rgb_to_jpeg(rgb_data: &[u8], width: u32, height: u32, quality: u8) -> PixieResult<Vec<u8>> {
    use image::{ImageBuffer, Rgb};
    
    let img = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb_data)
        .ok_or_else(|| PixieError::ProcessingError("Failed to create image buffer".to_string()))?;
    
    let mut buffer = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality.min(100));
    img.write_with_encoder(encoder)
        .map_err(|e| PixieError::ProcessingError(format!("JPEG encoding failed: {}", e)))?;
    
    Ok(buffer)
}
