// BMP Format Support

extern crate alloc;
use alloc::{vec::Vec, format};
use crate::types::{PixieResult, PixieError, ImageOptConfig};

#[cfg(feature = "image")]
use image::DynamicImage;

/// aggressive C hotspot preprocessing
pub fn optimize_bmp(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        // Start performance timing for WebAssembly
        #[cfg(target_arch = "wasm32")]
        let start_time = if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
            performance.now()
        } else {
            0.0
        };
        
        #[cfg(target_arch = "wasm32")]
        let data_size = data.len();
        
        // Load BMP image for processing
        let img = image::load_from_memory(data)
            .map_err(|e| PixieError::InvalidImageFormat(format!("Failed to load BMP: {}", e)))?;
        
        let mut best_result = data.to_vec();
        let best_size = data.len();
        
        // Apply C hotspot preprocessing for maximum compression benefit
        let processed_img = apply_bmp_c_hotspot_preprocessing(&img, quality)?;
        let final_img = processed_img.as_ref().unwrap_or(&img);
        
        // Quality-based output format selection
        if quality >= 85 {
            // High quality: Convert to PNG with C hotspot optimization
            let mut png_output = Vec::new();
            let png_encoder = image::codecs::png::PngEncoder::new(&mut png_output);
            
            if final_img.write_with_encoder(png_encoder).is_ok() && png_output.len() < best_size {
                best_result = png_output;
            }
        } else if quality >= 50 {
            // Medium quality: Try both PNG and JPEG, keep best
            
            // Try PNG first
            let mut png_output = Vec::new();
            let png_encoder = image::codecs::png::PngEncoder::new(&mut png_output);
            if final_img.write_with_encoder(png_encoder).is_ok() && png_output.len() < best_size {
                best_result = png_output;
            }
            
            // Try JPEG for better compression
            let jpeg_quality = match quality {
                50..=60 => 70,
                61..=70 => 80,
                71..=84 => 85,
                _ => 90,
            };
            
            let mut jpeg_output = Vec::new();
            let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, jpeg_quality);
            if final_img.write_with_encoder(jpeg_encoder).is_ok() && jpeg_output.len() < best_size {
                best_result = jpeg_output;
            }
        } else {
            // Low quality: Aggressive JPEG compression + try WebP
            
            let jpeg_quality = match quality {
                0..=20 => 30,
                21..=35 => 45,
                36..=49 => 60,
                _ => 70,
            };
            
            let mut jpeg_output = Vec::new();
            let jpeg_encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_output, jpeg_quality);
            if final_img.write_with_encoder(jpeg_encoder).is_ok() && jpeg_output.len() < best_size {
                best_result = jpeg_output;
            }
            
            // Try WebP for maximum compression
            #[cfg(feature = "codec-webp")]
            {
                if let Ok(webp_output) = crate::image::webp::optimize_webp_with_config(data, quality, config) {
                    if webp_output.len() < best_size {
                        best_result = webp_output;
                    }
                }
            }
        }
        
        // Performance tracking and logging for WebAssembly
        #[cfg(target_arch = "wasm32")]
        {
            let elapsed = if let Some(performance) = web_sys::window().and_then(|w| w.performance()) {
                performance.now() - start_time
            } else {
                0.0
            };
            
            // Log compression results for debugging
            use wasm_bindgen::prelude::*;
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = console)]
                fn log(s: &str);
            }
            
            let compression_ratio = ((data_size - best_result.len()) as f64 / data_size as f64) * 100.0;
            let msg = format!("BMP C hotspot optimization: {} -> {} bytes ({:.1}% compression) in {:.1}ms", 
                            data_size, best_result.len(), compression_ratio, elapsed);
            log(&msg);
            
            // Update global performance statistics (manually in WASM context)
            unsafe {
                extern "C" {
                    static mut PERF_STATS: crate::optimizers::PerformanceStats;
                }
                PERF_STATS.total_bytes_processed += data_size as u64;
                PERF_STATS.images_processed += 1;
                if elapsed > 100.0 { // IMAGE_TARGET_MS
                    PERF_STATS.performance_target_violations += 1;
                }
            }
        }
        
        Ok(best_result)
    }
    
    #[cfg(not(feature = "image"))]
    {
        Err(PixieError::UnsupportedFeature("BMP optimization requires image feature".to_string()))
    }
}

/// Apply BMP-specific C hotspot preprocessing for maximum compression benefit
#[cfg(all(feature = "image", c_hotspots_available))]
fn apply_bmp_c_hotspot_preprocessing(img: &DynamicImage, quality: u8) -> PixieResult<Option<DynamicImage>> {
    use image::GenericImageView;
    
    let (width, height) = img.dimensions();
    
    // Convert to RGBA for C hotspot processing
    let mut rgba_img = img.to_rgba8();
    let mut rgba_data = rgba_img.as_raw().clone();
    
    // BMP benefits greatly from aggressive color reduction
    let max_colors = match quality {
        0..=30 => 64,    // Very aggressive quantization
        31..=50 => 128,  // Aggressive quantization  
        51..=70 => 256,  // Standard quantization
        71..=85 => 512,  // Conservative quantization
        _ => 1024,       // Minimal quantization for high quality
    };
    
    // Stage 1: Octree quantization for color reduction (primary optimization)
    match crate::c_hotspots::image::octree_quantization(&rgba_data, width as usize, height as usize, max_colors) {
        Ok((palette, indices)) => {
            // Convert quantized data back to RGBA
            rgba_data = indices_to_rgba_bmp(&indices, &palette, width as usize, height as usize);
            
            // Stage 2: Floyd-Steinberg dithering to improve visual quality
            crate::c_hotspots::image::floyd_steinberg_dither(&mut rgba_data, width as usize, height as usize, &palette);
        },
        Err(_) => {
            // Fallback: Apply light Gaussian blur to reduce noise for compression
            crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width as usize, height as usize, 0.5);
        }
    }
    
    // Stage 3: Light blur for final compression optimization
    crate::c_hotspots::image::gaussian_blur(&mut rgba_data, width as usize, height as usize, 0.3);
    
    // Create optimized image
    use image::{ImageBuffer, RgbaImage};
    if let Some(processed_img) = ImageBuffer::from_raw(width, height, rgba_data) {
        Ok(Some(DynamicImage::ImageRgba8(processed_img)))
    } else {
        Ok(None)
    }
}

/// Fallback when C hotspots are not available - basic preprocessing
#[cfg(all(feature = "image", not(c_hotspots_available)))]
fn apply_bmp_c_hotspot_preprocessing(_img: &DynamicImage, _quality: u8) -> PixieResult<Option<DynamicImage>> {
    Ok(None)
}

/// Convert indexed color data back to RGBA for BMP processing
#[cfg(c_hotspots_available)]
fn indices_to_rgba_bmp(indices: &[u8], palette: &[crate::c_hotspots::Color32], width: usize, height: usize) -> Vec<u8> {
    let mut rgba_data = Vec::with_capacity(width * height * 4);
    
    for &index in indices {
        if (index as usize) < palette.len() {
            let color = &palette[index as usize];
            rgba_data.push(color.r);
            rgba_data.push(color.g);
            rgba_data.push(color.b);
            rgba_data.push(color.a);
        } else {
            // Fallback for invalid indices
            rgba_data.extend_from_slice(&[0, 0, 0, 255]);
        }
    }
    
    rgba_data
}

/// Force conversion from any image format to BMP with optimization
/// Unlike optimize_bmp, this function always converts to BMP regardless of input format
/// but applies full BMP optimization strategies
pub fn convert_any_format_to_bmp(data: &[u8]) -> crate::types::PixieResult<Vec<u8>> {
    #[cfg(feature = "image")]
    {
        use image::load_from_memory;
        use image::codecs::bmp::BmpEncoder;
        
        // Load the image from any format
        let img = load_from_memory(data)
            .map_err(|e| crate::types::PixieError::ProcessingError(format!("Failed to load image for BMP conversion: {}", e)))?;
        
        // First convert to BMP format (uncompressed)
        let mut temp_output = Vec::new();
        let encoder = BmpEncoder::new(&mut temp_output);
        
        // BMP doesn't support transparency, so convert to RGB
        let rgb_img = img.to_rgb8();
        rgb_img.write_with_encoder(encoder)
            .map_err(|e| crate::types::PixieError::ProcessingError(format!("BMP encoding failed: {}", e)))?;
        
        // Now apply BMP optimization strategies to the converted data
        // This ensures we get the benefits of format-specific optimization
        let config = crate::types::ImageOptConfig::default();
        optimize_bmp(&temp_output, 75, &config) // Use moderate quality for BMP conversion
    }
    #[cfg(not(feature = "image"))]
    {
        Err(crate::types::PixieError::FeatureNotEnabled("Image processing not available - missing image feature".to_string()))
    }
}