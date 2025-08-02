//! zune-png optimization implementation
//! 
//! High-performance PNG processing using the zune-png crate for better
//! performance compared to the generic image crate.

extern crate alloc;
use alloc::{vec::Vec, string::ToString};

use crate::types::{PixieResult, ImageOptConfig, PixieError};

#[cfg(feature = "codec-png")]
use zune_png::{PngDecoder, PngEncoder, zune_core::options::{DecoderOptions, EncoderOptions}};

/// Optimize PNG using zune-png for better performance
#[cfg(feature = "codec-png")]
pub fn optimize_png_zune(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate PNG signature
    if data.len() < 8 || !data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Err(PixieError::InvalidFormat("Not a valid PNG file".to_string()));
    }
    
    // For lossless mode, use metadata stripping only
    if config.lossless {
        return optimize_png_metadata_only(data, quality);
    }
    
    // Create decoder with optimized options
    let decoder_options = DecoderOptions::default();
    let mut decoder = PngDecoder::new_with_options(decoder_options);
    
    // Decode the PNG
    let (image_data, info) = decoder.decode_raw(data)
        .map_err(|e| PixieError::ProcessingError(format!("PNG decode failed: {}", e)))?;
    
    // Get image properties
    let width = info.width as usize;
    let height = info.height as usize;
    let color_type = info.color_type;
    let bit_depth = info.bit_depth;
    
    // Re-encode with optimized settings
    optimize_png_reencode(&image_data, width, height, color_type, bit_depth, quality, config)
}

/// Re-encode PNG data with optimized compression
#[cfg(feature = "codec-png")]
fn optimize_png_reencode(
    image_data: &[u8], 
    width: usize, 
    height: usize, 
    color_type: zune_png::zune_core::colorspace::ColorSpace,
    bit_depth: u8,
    quality: u8, 
    config: &ImageOptConfig
) -> PixieResult<Vec<u8>> {
    let mut encoder_options = EncoderOptions::default();
    
    // Set compression level based on quality
    let compression_level = match quality {
        0..=20 => 9,   // Maximum compression
        21..=40 => 8,  // High compression
        41..=60 => 6,  // Medium compression
        61..=80 => 4,  // Moderate compression
        81..=100 => 2, // Light compression
        _ => 6,
    };
    
    encoder_options.set_compression_level(compression_level);
    
    // Configure filtering strategy based on quality
    if quality < 50 {
        // More aggressive filtering for better compression
        encoder_options.set_filter(zune_png::zune_core::options::FilterType::All);
    } else {
        // Faster filtering for higher quality
        encoder_options.set_filter(zune_png::zune_core::options::FilterType::Paeth);
    }
    
    let mut encoder = PngEncoder::new_with_options(encoder_options);
    encoder.set_colorspace(color_type);
    encoder.set_depth(bit_depth);
    
    let encoded = encoder.encode(image_data, width, height)
        .map_err(|e| PixieError::ProcessingError(format!("PNG encode failed: {}", e)))?;
    
    // Try post-processing optimization
    if quality < 70 {
        optimize_png_postprocess(&encoded, quality)
    } else {
        Ok(encoded)
    }
}

/// Post-process PNG to remove unnecessary chunks
#[cfg(feature = "codec-png")]
fn optimize_png_postprocess(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let mut result = Vec::with_capacity(data.len());
    let mut pos = 8; // Skip PNG signature
    
    // Keep PNG signature
    result.extend_from_slice(&data[0..8]);
    
    while pos + 8 < data.len() {
        let chunk_len = u32::from_be_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;
        
        if pos + 8 + chunk_len + 4 > data.len() {
            break;
        }
        
        let chunk_type = &data[pos + 4..pos + 8];
        let chunk_full = &data[pos..pos + 8 + chunk_len + 4];
        
        // Decide whether to keep chunk based on quality and type
        let keep_chunk = match chunk_type {
            b"IHDR" | b"IDAT" | b"IEND" => true, // Critical chunks
            b"PLTE" | b"tRNS" => true, // Essential for color/transparency
            b"bKGD" | b"pHYs" => quality > 40, // Background, dimensions
            b"gAMA" | b"cHRM" | b"sRGB" | b"iCCP" => quality > 60, // Color space
            b"tIME" => quality > 80, // Timestamp
            b"tEXt" | b"zTXt" | b"iTXt" => quality > 90, // Text metadata
            _ => false, // Unknown chunks
        };
        
        if keep_chunk {
            result.extend_from_slice(chunk_full);
        }
        
        pos += 8 + chunk_len + 4;
    }
    
    Ok(result)
}

/// Optimize PNG by stripping metadata only (lossless)
fn optimize_png_metadata_only(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let mut result = Vec::with_capacity(data.len());
    let mut pos = 8; // Skip PNG signature
    
    // Keep PNG signature
    result.extend_from_slice(&data[0..8]);
    
    while pos + 8 < data.len() {
        if pos + 4 > data.len() {
            break;
        }
        
        let chunk_len = u32::from_be_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;
        
        if pos + 8 + chunk_len + 4 > data.len() {
            break;
        }
        
        let chunk_type = &data[pos + 4..pos + 8];
        let chunk_full = &data[pos..pos + 8 + chunk_len + 4];
        
        // Apply quality-based filtering for metadata chunks
        let keep_chunk = match chunk_type {
            b"IHDR" | b"IDAT" | b"IEND" => true, // Critical chunks
            b"PLTE" | b"tRNS" => true, // Always keep palette and transparency
            b"bKGD" | b"pHYs" => quality > 60, // Background, dimensions
            b"gAMA" | b"cHRM" | b"sRGB" | b"iCCP" => quality > 80, // Color space
            b"tIME" => quality > 90, // Timestamp - only for highest quality
            b"tEXt" | b"zTXt" | b"iTXt" => quality > 95, // Text metadata
            _ => false, // Remove unknown chunks
        };
        
        if keep_chunk {
            result.extend_from_slice(chunk_full);
        }
        
        pos += 8 + chunk_len + 4;
    }
    
    Ok(result)
}

/// Fallback to basic PNG optimization when zune-png is not available
#[cfg(not(feature = "codec-png"))]
pub fn optimize_png_zune(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Fall back to the basic PNG optimization from png.rs
    crate::image::png::optimize_png_rust(data, quality)
}
