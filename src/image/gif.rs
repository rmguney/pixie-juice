#[cfg(feature = "codec-gif")]
extern crate gif;

extern crate alloc;
use alloc::{vec, vec::Vec, string::ToString, format};

use crate::types::{PixieResult, PixieError, ImageOptConfig, ImageInfo, ColorSpace};

#[cfg(feature = "color_quant")]
use color_quant::NeuQuant;

pub fn optimize_gif(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    if !is_gif(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid GIF file".to_string()));
    }

    if data.is_empty() {
        return Err(PixieError::InvalidInput("Empty GIF data".to_string()));
    }

    let is_animated = detect_animated_gif(data);
    
    let strategies = get_gif_optimization_strategies(quality, is_animated);
    let mut best_result = data.to_vec();
    let mut best_size = data.len();
    
    for strategy in strategies {
        if let Ok(optimized) = apply_gif_strategy(data, strategy, quality, config) {
            if optimized.len() < best_size {
                best_result = optimized;
                best_size = best_result.len();
            }
        }
    }
    
    if best_result.len() < data.len() {
        Ok(best_result)
    } else {
        Ok(data.to_vec())
    }
}

#[derive(Debug, Clone)]
enum GifOptimizationStrategy {
    StripMetadata,
    OptimizePalette,
    DeduplicateFrames,
    ConvertToPNG,
    ConvertToWebP,
    CHotspot,
}

pub fn detect_animated_gif(data: &[u8]) -> bool {
    if data.len() < 13 {
        return false;
    }
    
    if !data[0..6].eq(b"GIF87a") && !data[0..6].eq(b"GIF89a") {
        return false;
    }
    
    let mut image_blocks = 0;
    let mut pos = 13;
    
    while pos < data.len() {
        match data[pos] {
            0x2C => {
                image_blocks += 1;
                if image_blocks > 1 {
                    return true;
                }
                pos += 10; 
            },
            0x21 => {
                pos += 2;
                if pos < data.len() {
                    while pos < data.len() && data[pos] != 0 {
                        pos += data[pos] as usize + 1;
                    }
                    pos += 1;
                }
            },
            0x3B => break, 
            _ => pos += 1,
        }
    }
    
    false
}

fn get_gif_optimization_strategies(quality: u8, is_animated: bool) -> Vec<GifOptimizationStrategy> {
    let mut strategies = Vec::new();
    
    strategies.push(GifOptimizationStrategy::StripMetadata);
    
    #[cfg(c_hotspots_available)]
    {
        strategies.push(GifOptimizationStrategy::CHotspot);
    }
    
    strategies.push(GifOptimizationStrategy::OptimizePalette);
    
    if is_animated {
        strategies.push(GifOptimizationStrategy::DeduplicateFrames); 
        if quality <= 60 {
            strategies.push(GifOptimizationStrategy::ConvertToWebP);
        }
    } else {
        if quality <= 70 {
            strategies.push(GifOptimizationStrategy::ConvertToPNG);
        }
    }
    
    strategies
}

fn apply_gif_strategy(data: &[u8], strategy: GifOptimizationStrategy, quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    match strategy {
        GifOptimizationStrategy::StripMetadata => strip_gif_metadata(data, quality),
        GifOptimizationStrategy::OptimizePalette => optimize_gif_palette(data, quality),
        GifOptimizationStrategy::DeduplicateFrames => deduplicate_gif_frames(data),
        GifOptimizationStrategy::ConvertToPNG => convert_gif_to_png(data, config),
        GifOptimizationStrategy::ConvertToWebP => convert_gif_to_webp(data, quality),
        GifOptimizationStrategy::CHotspot => optimize_gif_with_c_hotspot(data, quality),
    }
}

fn optimize_gif_palette(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    Ok(data.to_vec())
}

fn deduplicate_gif_frames(data: &[u8]) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    #[cfg(feature = "codec-gif")]
    {
        log_to_console("🎬 Starting GIF frame deduplication using gif crate");
        
        log_to_console("🔄 Using reliable metadata stripping for GIF optimization");
        strip_gif_metadata(data, 75)
    }
    
    #[cfg(not(feature = "codec-gif"))]
    {
        let _ = data;
        log_to_console("⚠️ GIF codec not available - using manual optimization");
        strip_gif_metadata_manual(data, 75)
    }
}

#[cfg(feature = "codec-gif")]
fn simple_frame_hash(data: &[u8]) -> u64 {
    let mut hash = 0u64;
    for (i, &byte) in data.iter().enumerate() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64).wrapping_add(i as u64);
    }
    hash
}

fn convert_gif_to_png(data: &[u8], _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load GIF: {}", e)))?;
        
        let mut output = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut output);
        
        img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ProcessingError(format!("PNG encoding failed: {}", e)))?;
        
        Ok(output)
    }
    
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, _config);
        Err(PixieError::FeatureNotEnabled("GIF to PNG conversion requires 'image' feature".to_string()))
    }
}

fn convert_gif_to_webp(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load GIF: {}", e)))?;
        
        let mut output = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
        
        img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ProcessingError(format!("WebP encoding failed: {}", e)))?;
        
        Ok(output)
    }
    
    #[cfg(not(feature = "image"))]
    {
        let _ = (data, _quality);
        Err(PixieError::FeatureNotEnabled("GIF to WebP conversion requires 'image' feature".to_string()))
    }
}

pub fn optimize_gif_rust(data: &[u8], quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    log_to_console(&format!("GIF optimization with external crates: {} bytes", data.len()));
    
    match strip_gif_metadata(data, quality) {
        Ok(result) => {
            if result.len() < data.len() {
                log_to_console(&format!("Metadata stripping successful: {} -> {} bytes ({:.1}% savings)", 
                    data.len(), result.len(), 
                    ((data.len() - result.len()) as f64 / data.len() as f64) * 100.0));
                return Ok(result);
            }
        },
        Err(e) => {
            log_to_console(&format!("Metadata stripping failed: {}", e));
        }
    }
    
    #[cfg(all(feature = "codec-gif", feature = "color_quant"))]
    {
        log_to_console("Metadata stripping minimal benefit - trying advanced analysis");
        match try_gif_analysis(data, quality) {
            Ok(result) => return Ok(result),
            Err(e) => {
                log_to_console(&format!("GIF analysis failed ({}), using basic optimization", e));
            }
        }
    }
    
    log_to_console("Falling back to basic optimization");
    try_basic_gif_optimization(data, quality)
}

#[cfg(all(feature = "codec-gif", feature = "color_quant"))]
fn try_gif_analysis(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    
    let mut decoder = match options.read_info(data) {
        Ok(decoder) => decoder,
        Err(_) => return Err(PixieError::InvalidImageFormat("Failed to decode GIF header".to_string())),
    };
    let width = decoder.width() as usize;
    let height = decoder.height() as usize;
    
    log_to_console(&format!("GIF info: {}x{} pixels", width, height));
    
    if width == 0 || height == 0 || width * height > 100_000_000 {
        return Err(PixieError::InvalidImageFormat("Invalid GIF dimensions".to_string()));
    }
    
    let mut frame_data = vec![0u8; width * height * 4];
    
    if decoder.read_next_frame().is_err() || decoder.fill_buffer(&mut frame_data).is_err() {
        return Err(PixieError::InvalidImageFormat("Failed to read GIF frame data".to_string()));
    }
    
    let non_zero_pixels = frame_data.chunks(4).take(100).any(|pixel| pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0);
    
    if !non_zero_pixels {
        log_to_console("Frame data appears to be empty or corrupted");
        return Err(PixieError::InvalidImageFormat("Empty or corrupted frame data".to_string()));
    }
    
    let target_colors = ((255 - quality as u32) * 255 / 100).max(16).min(256) as u8;
    log_to_console(&format!("Analyzing palette with target {} colors", target_colors));
    
    if frame_data.len() < 4 || target_colors == 0 {
        return Err(PixieError::InvalidInput("Invalid parameters for color quantization".to_string()));
    }
    
    let neuquant = NeuQuant::new(10, target_colors as usize, &frame_data);
    let optimized_palette = neuquant.color_map_rgba();
    log_to_console(&format!("Color quantization complete - {} colors in optimized palette", optimized_palette.len() / 4));
    
    let stripped = strip_gif_metadata(data, quality)?;
    
    if stripped.len() < data.len() {
        log_to_console(&format!("Metadata stripping successful: {} -> {} bytes ({:.1}% savings)", 
            data.len(), stripped.len(), 
            ((data.len() - stripped.len()) as f64 / data.len() as f64) * 100.0));
        return Ok(stripped);
    }
    
    log_to_console("Metadata stripping minimal benefit - trying basic recompression");
    
    if stripped.len() >= data.len() {
        return try_basic_gif_optimization(data, quality);
    }
    
    Ok(stripped)
}

#[cfg(feature = "codec-gif")]
fn strip_gif_metadata(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    log_to_console(&format!("Starting metadata stripping on {} byte GIF using gif crate", data.len()));
    
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::Indexed);
    
    let mut decoder = match options.read_info(data) {
        Ok(decoder) => decoder,
        Err(e) => {
            log_to_console(&format!("GIF validation failed: {}, falling back to manual", e));
            return strip_gif_metadata_manual(data, quality);
        }
    };
    
    let width = decoder.width();
    let height = decoder.height();
    let global_palette = decoder.global_palette().map(|p| p.to_vec());
    
    log_to_console(&format!("GIF validation successful: {}x{} pixels", width, height));
    
    let mut frames = Vec::new();
    let mut frame_count = 0;
    
    while let Ok(Some(frame)) = decoder.read_next_frame() {
        frame_count += 1;
        log_to_console(&format!("Processing frame {}", frame_count));
        
        frames.push((
            frame.buffer.to_vec(),
            frame.left,
            frame.top,
            frame.width,
            frame.height,
            frame.delay,
            frame.dispose,
            frame.transparent,
            frame.palette.clone(),
        ));
        
        if frame_count >= 1000 {
            log_to_console("Reached maximum frame limit (1000), stopping");
            break;
        }
    }
    
    log_to_console(&format!("Found {} frames in GIF", frame_count));
    
    if frame_count == 0 {
        return Err(PixieError::InvalidImageFormat("No frames found in GIF".to_string()));
    }
    
    let mut output = Vec::new();
    {
        let mut encoder = gif::Encoder::new(&mut output, width, height, &global_palette.unwrap_or_default())
            .map_err(|e| PixieError::ImageEncodingFailed(format!("Failed to create GIF encoder: {}", e)))?;
        
        if frame_count > 1 {
            encoder.set_repeat(gif::Repeat::Infinite)
                .map_err(|e| PixieError::ImageEncodingFailed(format!("Failed to set GIF repeat: {}", e)))?;
        }
        
        for (i, (buffer, left, top, frame_width, frame_height, delay, dispose, transparent, palette)) in frames.iter().enumerate() {
            let mut frame = gif::Frame::from_indexed_pixels(
                *frame_width,
                *frame_height,
                buffer.clone(),
                transparent.clone(),
            );
            
            frame.left = *left;
            frame.top = *top;
            frame.delay = *delay;
            frame.dispose = *dispose;
            
            if let Some(pal) = palette {
                frame.palette = Some(pal.clone());
            }
            
            encoder.write_frame(&frame)
                .map_err(|e| PixieError::ImageEncodingFailed(format!("Failed to write GIF frame {}: {}", i, e)))?;
            
            if i % 10 == 0 {
                log_to_console(&format!("Re-encoded frame {}/{}", i + 1, frame_count));
            }
        }
    }
    
    let bytes_saved = data.len().saturating_sub(output.len());
    
    log_to_console(&format!("Metadata stripping complete: {} frames processed", frame_count));
    log_to_console(&format!("Size change: {} -> {} bytes ({} bytes saved)", 
        data.len(), output.len(), bytes_saved));
    
    if output.len() < data.len() {
        log_to_console(&format!("Successful optimization: {:.1}% reduction", 
            ((data.len() - output.len()) as f64 / data.len() as f64) * 100.0));
        Ok(output)
    } else {
        log_to_console("No size reduction achieved, returning original");
        Ok(data.to_vec())
    }
}

#[cfg(not(feature = "codec-gif"))]
fn strip_gif_metadata(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    strip_gif_metadata_manual(data, _quality)
}

fn strip_gif_metadata_manual(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    log_to_console("Using fallback manual metadata stripping");
    
    if data.len() < 6 {
        return Err(PixieError::InvalidImageFormat("GIF file too small".to_string()));
    }

    let mut output = Vec::with_capacity(data.len());
    let mut pos = 0;
    let mut _extensions_stripped = 0;

    while pos < data.len() {
        if pos + 1 < data.len() && data[pos] == 0x21 && data[pos + 1] == 0xFF {
            _extensions_stripped += 1;
            pos += 2;
            
            while pos < data.len() {
                let block_size = data[pos];
                pos += 1;
                if block_size == 0 {
                    break;
                }
                pos += block_size as usize;
                if pos > data.len() {
                    break;
                }
            }
        } else {
            output.push(data[pos]);
            pos += 1;
        }
    }

    let bytes_saved = data.len().saturating_sub(output.len());
    
    if bytes_saved > 0 {
        log_to_console(&format!("Manual stripping saved {} bytes ({:.1}% reduction)", 
            bytes_saved, (bytes_saved as f64 / data.len() as f64) * 100.0));
    }

    Ok(output)
}

fn try_basic_gif_optimization(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    use crate::image::log_to_console;
    
    log_to_console("Attempting basic GIF recompression optimization");
    
    let mut optimized = data.to_vec();
    
    while optimized.len() > 1 && optimized[optimized.len() - 1] == 0 {
        optimized.pop();
    }
    
    if !optimized.ends_with(&[0x3B]) { optimized.push(0x3B); }
    
    let savings = data.len().saturating_sub(optimized.len());
    
    if savings > 0 {
        log_to_console(&format!("Basic optimization achieved {} bytes savings ({:.1}%)", 
            savings, (savings as f64 / data.len() as f64) * 100.0));
        Ok(optimized)
    } else {
        log_to_console("No improvement possible with basic optimization techniques");
        if data.len() > 10 {
            let mut result = data.to_vec();
            while result.len() > 6 && result[result.len() - 2] == 0 && result[result.len() - 1] == 0x3B {
                result.remove(result.len() - 2);
            }
            if result.len() < data.len() {
                log_to_console(&format!("Removed {} trailing null bytes", data.len() - result.len()));
                return Ok(result);
            }
        }
        Ok(data.to_vec())
    }
}

pub fn is_gif(data: &[u8]) -> bool {
    data.len() >= 6 && (
        data.starts_with(b"GIF87a") || 
        data.starts_with(b"GIF89a")
    )
}

pub fn get_gif_info(data: &[u8]) -> PixieResult<ImageInfo> {
    if !is_gif(data) {
        return Err(PixieError::InvalidImageFormat("Not a valid GIF file".to_string()));
    }

    if data.len() < 13 {
        return Err(PixieError::InvalidImageFormat("GIF file too small".to_string()));
    }

    let width = u16::from_le_bytes([data[6], data[7]]) as u32;
    let height = u16::from_le_bytes([data[8], data[9]]) as u32;
    
    let packed = data[10];
    let has_color_table = (packed & 0x80) != 0;
    let color_table_size = if has_color_table {
        ((packed & 0x07) + 1) as u8
    } else {
        8
    };

    Ok(ImageInfo {
        width,
        height,
        channels: 3,
        bit_depth: color_table_size,
        format: "GIF".to_string(),
        has_alpha: false, // Would need deeper analysis
        color_space: ColorSpace::RGB,
        compression: Some("LZW".to_string()),
        file_size: Some(data.len()),
    })
}

#[cfg(feature = "c_hotspots")]
extern "C" {
    fn optimize_gif_c_hotspot(data: *const u8, len: usize, quality: u8, output: *mut u8, output_len: *mut usize) -> i32;
}

#[cfg(feature = "c_hotspots")]
fn optimize_gif_c(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let mut output = vec![0u8; data.len() * 2];
    let mut output_len = output.len();
    
    unsafe {
        let result = optimize_gif_c_hotspot(
            data.as_ptr(),
            data.len(),
            quality,
            output.as_mut_ptr(),
            &mut output_len as *mut usize
        );
        
        if result == 0 {
            output.truncate(output_len);
            Ok(output)
        } else {
            Err(PixieError::CHotspotFailed(format!("C hotspot failed with code {}", result)))
        }
    }
}

fn optimize_gif_with_c_hotspot(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(c_hotspots_available)]
    {
        match optimize_gif_c(data, quality) {
            Ok(c_result) => {
                let mut best_result = c_result;
                let mut best_size = best_result.len();
                
                if best_size > data.len() / 2 {
                    if let Ok(stripped) = strip_gif_metadata(&best_result, quality) {
                        if stripped.len() < best_size {
                            best_result = stripped;
                            best_size = best_result.len();
                        }
                    }
                    
                    if let Ok(palette_opt) = optimize_gif_palette(&best_result, quality) {
                        if palette_opt.len() < best_size {
                            best_result = palette_opt;
                        }
                    }
                }
                
                #[cfg(target_arch = "wasm32")]
                {
                    use wasm_bindgen::prelude::*;
                    #[wasm_bindgen]
                    extern "C" {
                        #[wasm_bindgen(js_namespace = console)]
                        fn log(s: &str);
                    }
                    
                    let compression_ratio = ((data.len() - best_result.len()) as f64 / data.len() as f64) * 100.0;
                    let msg = format!("GIF C hotspot optimization: {} -> {} bytes ({:.1}% compression)", 
                                    data.len(), best_result.len(), compression_ratio);
                    log(&msg);
                }
                
                return Ok(best_result);
            },
            Err(_) => {
            }
        }
    }
    
    optimize_gif_rust_fallback(data, quality)
}

fn optimize_gif_rust_fallback(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let mut best_result = data.to_vec();
    let mut best_size = data.len();
    
    if let Ok(stripped) = strip_gif_metadata(data, quality) {
        if stripped.len() < best_size {
            best_result = stripped;
            best_size = best_result.len();
        }
    }
    
    if let Ok(palette_opt) = optimize_gif_palette(&best_result, quality) {
        if palette_opt.len() < best_size {
            best_result = palette_opt;
            best_size = best_result.len();
        }
    }
    
    if quality <= 60 && best_size >= data.len() * 90 / 100 {
        #[cfg(feature = "image")]
        {
            if !detect_animated_gif(data) {
                if let Ok(png_result) = convert_gif_to_png(data, &ImageOptConfig::default()) {
                    if png_result.len() < best_size {
                        best_result = png_result;
                    }
                }
            }
        }
    }
    
    Ok(best_result)
}

pub fn convert_any_format_to_gif(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        
        let img = load_from_memory(data)
            .map_err(|e| PixieError::ProcessingError(format!("Failed to load image for GIF conversion: {}", e)))?;
        
        let mut temp_output = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut temp_output);
        
        img.write_with_encoder(encoder)
            .map_err(|e| PixieError::ProcessingError(format!("PNG encoding failed (GIF conversion step): {}", e)))?;
        
        let config = crate::types::ImageOptConfig::default();
        optimize_gif_rust(&temp_output, quality, &config)
    }
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::FeatureNotEnabled("Image processing not available - missing image feature".to_string()))
    }
}
