//! PNG optimization using oxipng
//! 
//! oxipng is the best-in-class PNG optimizer, providing lossless compression
//! that often achieves better results than pngcrush, optipng, etc.

use crate::types::{OptConfig, OptError, OptResult};

/// Optimize PNG data using oxipng
pub fn optimize_png(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Create oxipng options based on configuration
    let mut options = oxipng::Options::default();
    
    // Set optimization based on quality/speed preference
    let fast_mode = config.fast_mode.unwrap_or(false);
    
    // Configure optimization settings
    options.optimize_alpha = !config.preserve_metadata.unwrap_or(false);
    options.strip = if config.preserve_metadata.unwrap_or(false) {
        oxipng::StripChunks::None
    } else {
        oxipng::StripChunks::Safe
    };

    // Set filter options
    if fast_mode {
        options.filter = [oxipng::RowFilter::None].iter().cloned().collect();
    } else {
        // Use multiple filters for better compression
        options.filter = [
            oxipng::RowFilter::None,
            oxipng::RowFilter::Sub,
            oxipng::RowFilter::Up,
            oxipng::RowFilter::Average,
            oxipng::RowFilter::Paeth,
        ].iter().cloned().collect();
    }

    // Perform optimization
    match oxipng::optimize_from_memory(data, &options) {
        Ok(optimized) => {
            log::info!("PNG optimized: {} bytes -> {} bytes ({:.1}% reduction)", 
                      data.len(), 
                      optimized.len(),
                      (1.0 - (optimized.len() as f64 / data.len() as f64)) * 100.0);
            Ok(optimized)
        },
        Err(e) => {
            log::warn!("PNG optimization failed: {}, returning original", e);
            // Return original data if optimization fails
            Ok(data.to_vec())
        }
    }
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

#[derive(Debug, Clone)]
pub struct PngInfo {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub channels: u8,
    pub file_size: usize,
}
