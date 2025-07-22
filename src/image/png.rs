//! PNG optimization using oxipng with C hotspot integration
//! C hotspots provide additional filtering and compression for edge cases.

use crate::types::{OptConfig, OptError, OptResult};
use image::RgbaImage;

#[cfg(feature = "c_hotspots")]
use crate::ffi::image_ffi::{quantize_colors_octree_safe};

/// Optimize PNG data using oxipng with aggressive settings and C hotspots
pub fn optimize_png(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Create oxipng options with aggressive optimization
    let mut options = oxipng::Options::default();
    
    // Set optimization based on quality/speed preference
    let fast_mode = config.fast_mode.unwrap_or(false);
    
    // Configure optimization settings for maximum compression
    options.optimize_alpha = true;
    options.strip = if config.preserve_metadata.unwrap_or(false) {
        oxipng::StripChunks::Safe  // Still strip some chunks
    } else {
        oxipng::StripChunks::All   // Strip everything for maximum compression
    };

    // Set deflate options for maximum compression - more aggressive defaults
    options.deflate = if fast_mode {
        oxipng::Deflaters::Libdeflater { compression: 8 }  // Higher than before
    } else {
        oxipng::Deflaters::Libdeflater { compression: 12 } // Maximum compression
    };
    
    // Set filter options for maximum compression - always use all filters
    options.filter = [
        oxipng::RowFilter::None,
        oxipng::RowFilter::Sub,
        oxipng::RowFilter::Up,
        oxipng::RowFilter::Average,
        oxipng::RowFilter::Paeth,
        oxipng::RowFilter::MinSum,
    ].iter().cloned().collect();

    // Enable aggressive optimization features
    options.interlace = Some(oxipng::Interlacing::None); // Remove interlacing
    options.palette_reduction = true;
    options.grayscale_reduction = true;
    options.bit_depth_reduction = true;
    options.color_type_reduction = true;
    
    // More aggressive settings based on target reduction
    if let Some(target_reduction) = config.target_reduction {
        if target_reduction >= 0.4 { // 40%+ reduction target
            // Use maximum compression settings with multiple passes
            options.deflate = oxipng::Deflaters::Libdeflater { compression: 12 };
        }
    } else {
        // Default to very aggressive optimization
        options.deflate = oxipng::Deflaters::Libdeflater { compression: 11 };
    }

    // First optimization attempt
    let mut best_result = data.to_vec();
    let mut best_reduction = 0.0;

    // Try multiple optimization strategies
    let strategies = vec![
        // Strategy 1: Maximum compression
        {
            let mut opts = options.clone();
            opts.deflate = oxipng::Deflaters::Libdeflater { compression: 12 };
            opts
        },
        // Strategy 2: Balanced with different filters
        {
            let mut opts = options.clone();
            opts.deflate = oxipng::Deflaters::Libdeflater { compression: 9 };
            opts.filter = [oxipng::RowFilter::MinSum, oxipng::RowFilter::Paeth].iter().cloned().collect();
            opts
        },
        // Strategy 3: Focus on bit depth reduction
        {
            let mut opts = options.clone();
            opts.bit_depth_reduction = true;
            opts.palette_reduction = true;
            opts.grayscale_reduction = true;
            opts
        }
    ];

    for (i, strategy) in strategies.iter().enumerate() {
        match oxipng::optimize_from_memory(data, strategy) {
            Ok(optimized) => {
                let reduction = 1.0 - (optimized.len() as f64 / data.len() as f64);
                log::info!("PNG strategy {} result: {} bytes -> {} bytes ({:.1}% reduction)", 
                          i + 1, data.len(), optimized.len(), reduction * 100.0);
                
                if reduction > best_reduction {
                    best_result = optimized;
                    best_reduction = reduction;
                }
            },
            Err(e) => {
                log::debug!("PNG strategy {} failed: {}", i + 1, e);
            }
        }
    }

    // If we achieved good reduction, return the best result
    if best_reduction > 0.01 { // At least 1% reduction
        log::info!("PNG optimized with best reduction: {:.1}%", best_reduction * 100.0);
        return Ok(best_result);
    }

    // If standard optimization didn't work well, try C hotspots for edge cases
    if !config.preserve_metadata.unwrap_or(false) && !fast_mode {
        log::info!("Applying C hotspot PNG optimization for edge cases...");
        
        #[cfg(feature = "c_hotspots")]
        {
            if let Ok(hotspot_optimized) = try_c_hotspot_png_optimization(data, config) {
                let hotspot_reduction = 1.0 - (hotspot_optimized.len() as f64 / data.len() as f64);
                if hotspot_reduction > best_reduction {
                    log::info!("C hotspot optimization successful: {} bytes ({:.1}% total reduction)", 
                              hotspot_optimized.len(), hotspot_reduction * 100.0);
                    return Ok(hotspot_optimized);
                }
            }
        }
    }

    // Return the best result we found, even if it's the original
    log::info!("PNG optimization completed with {:.1}% reduction", best_reduction * 100.0);
    Ok(best_result)
}

/// Check if PNG can be optimized further
pub fn can_optimize_png(data: &[u8]) -> bool {
    // Quick check: if file is very small, optimization likely won't help much
    if data.len() < 1024 {
        return false;
    }

    // Check if it looks like PNG format
    data.len() >= 8 && data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
}

/// Get PNG-specific information
pub fn get_png_info(data: &[u8]) -> OptResult<PngInfo> {
    if !can_optimize_png(data) {
        return Err(OptError::InvalidFormat("Not a valid PNG file".to_string()));
    }

    // Parse PNG header to get basic information
    // PNG structure: 8-byte signature + chunks
    // IHDR chunk is always first: length(4) + type(4) + data(13) + CRC(4)
    if data.len() < 33 {
        return Err(OptError::InvalidFormat("PNG file too small".to_string()));
    }

    let ihdr_start = 8; // Skip PNG signature
    let width = u32::from_be_bytes([data[ihdr_start + 8], data[ihdr_start + 9], 
                                   data[ihdr_start + 10], data[ihdr_start + 11]]);
    let height = u32::from_be_bytes([data[ihdr_start + 12], data[ihdr_start + 13], 
                                    data[ihdr_start + 14], data[ihdr_start + 15]]);
    let bit_depth = data[ihdr_start + 16];
    let color_type = data[ihdr_start + 17];

    let channels = match color_type {
        0 => 1, // Grayscale
        2 => 3, // RGB
        3 => 1, // Palette (indexed)
        4 => 2, // Grayscale + Alpha
        6 => 4, // RGB + Alpha
        _ => return Err(OptError::InvalidFormat("Invalid PNG color type".to_string())),
    };

    Ok(PngInfo {
        width,
        height,
        bit_depth,
        color_type,
        channels,
        file_size: data.len(),
    })
}

#[cfg(feature = "c_hotspots")]
fn try_c_hotspot_png_optimization(data: &[u8], _config: &OptConfig) -> OptResult<Vec<u8>> {
    use image::ImageFormat;
    
    // Decode PNG to raw RGBA data
    let img = image::load_from_memory_with_format(data, ImageFormat::Png)
        .map_err(|e| OptError::ProcessingError(format!("Failed to decode PNG: {}", e)))?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    
    // Try color quantization with C hotspots for PNGs with many colors
    let unique_colors = count_unique_colors(&rgba_img);
    if unique_colors > 256 {
        log::info!("PNG has {} unique colors, applying C hotspot quantization", unique_colors);
        
        // Apply octree quantization to reduce colors
        if let Ok(quantized) = quantize_colors_octree_safe(
            rgba_img.as_raw(),
            width,
            height,
            256 // Reduce to 256 colors max
        ) {
            log::info!("Applied C hotspot color quantization: {} -> {} colors", 
                      unique_colors, quantized.palette.len());
            
            // Convert quantized data back to RGBA for PNG encoding
            let mut quantized_rgba = Vec::with_capacity((width * height * 4) as usize);
            let default_color = crate::ffi::image_ffi::Color32::rgb(0, 0, 0);
            for &index in &quantized.data {
                let color = quantized.palette.get(index as usize).unwrap_or(&default_color);
                quantized_rgba.extend_from_slice(&[color.r, color.g, color.b, color.a]);
            }
            
            // Create new image from quantized data
            if let Some(quantized_img) = image::RgbaImage::from_raw(width, height, quantized_rgba) {
                // Encode to PNG with palette optimization
                let mut png_data = Vec::new();
                {
                    use std::io::Cursor;
                    
                    let dynamic_img = image::DynamicImage::ImageRgba8(quantized_img);
                    let mut cursor = Cursor::new(&mut png_data);
                    
                    if dynamic_img.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
                        drop(cursor); // Ensure cursor is dropped before using png_data
                        
                        // Run through oxipng for final optimization
                        let mut options = oxipng::Options::default();
                        options.palette_reduction = true;
                        options.bit_depth_reduction = true;
                        options.color_type_reduction = true;
                        options.optimize_alpha = true;
                        options.strip = oxipng::StripChunks::All;
                        options.deflate = oxipng::Deflaters::Libdeflater { compression: 12 };
                        
                        if let Ok(final_optimized) = oxipng::optimize_from_memory(&png_data, &options) {
                            let final_reduction = 1.0 - (final_optimized.len() as f64 / data.len() as f64);
                            if final_reduction > 0.05 { // At least 5% reduction to be worthwhile
                                log::info!("C hotspot quantization + oxipng: {:.1}% total reduction", final_reduction * 100.0);
                                return Ok(final_optimized);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // If quantization didn't help much, try other C hotspot optimizations
    // For now, just return original data
    Ok(data.to_vec())
}

#[allow(dead_code)]
fn count_unique_colors(img: &RgbaImage) -> usize {
    use std::collections::HashSet;
    let mut colors = HashSet::new();
    for pixel in img.pixels() {
        colors.insert((pixel[0], pixel[1], pixel[2], pixel[3]));
    }
    colors.len()
}

#[derive(Debug, Clone)]
pub struct PngInfo {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub channels: u8,
    pub file_size: usize,
}
