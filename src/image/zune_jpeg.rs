//! zune-jpeg optimization implementation
//! 
//! High-performance JPEG processing using the zune-jpeg crate for better
//! performance compared to the generic image crate.

extern crate alloc;
use alloc::{vec::Vec, string::ToString};

use crate::types::{PixieResult, ImageOptConfig, PixieError};

#[cfg(feature = "codec-jpeg")]
use zune_jpeg::{JpegDecoder, zune_core::options::DecoderOptions};

/// Optimize JPEG using zune-jpeg for better performance
#[cfg(feature = "codec-jpeg")]
pub fn optimize_jpeg_zune(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate JPEG signature
    if data.len() < 2 || !data.starts_with(&[0xFF, 0xD8]) {
        return Err(PixieError::InvalidFormat("Not a valid JPEG file".to_string()));
    }
    
    // For lossless mode, use metadata stripping only
    if config.lossless {
        return optimize_jpeg_metadata_only(data, quality);
    }
    
    // Create decoder with optimized options
    let mut decoder_options = DecoderOptions::default();
    decoder_options.jpeg_set_out_colorspace(zune_jpeg::zune_core::colorspace::ColorSpace::RGB);
    
    let mut decoder = JpegDecoder::new_with_options(decoder_options);
    decoder.decode_headers(data)
        .map_err(|e| PixieError::ProcessingError(format!("JPEG header decode failed: {}", e)))?;
    
    // Get image dimensions and decode
    let (width, height) = decoder.dimensions()
        .ok_or_else(|| PixieError::ProcessingError("Could not get JPEG dimensions".to_string()))?;
    
    let rgb_data = decoder.decode(data)
        .map_err(|e| PixieError::ProcessingError(format!("JPEG decode failed: {}", e)))?;
    
    // Re-encode with optimized quality
    optimize_jpeg_reencode(&rgb_data, width, height, quality, config)
}

/// Re-encode RGB data as optimized JPEG
#[cfg(feature = "codec-jpeg")]
fn optimize_jpeg_reencode(rgb_data: &[u8], width: usize, height: usize, quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    use zune_jpeg::{JpegEncoder, zune_core::options::EncoderOptions};
    
    // Create encoder with quality settings
    let mut encoder_options = EncoderOptions::default();
    encoder_options.set_quality(quality);
    
    // Enable progressive encoding for better compression
    if quality < 80 {
        encoder_options.set_progressive(true);
    }
    
    // Optimize chroma subsampling based on quality
    if quality < 60 {
        encoder_options.set_chroma_subsampling(zune_jpeg::zune_core::options::ChromaSubsampling::CS_420);
    } else if quality < 85 {
        encoder_options.set_chroma_subsampling(zune_jpeg::zune_core::options::ChromaSubsampling::CS_422);
    } else {
        encoder_options.set_chroma_subsampling(zune_jpeg::zune_core::options::ChromaSubsampling::CS_444);
    }
    
    let mut encoder = JpegEncoder::new_with_options(encoder_options);
    encoder.set_colorspace(zune_jpeg::zune_core::colorspace::ColorSpace::RGB);
    
    let encoded = encoder.encode(rgb_data, width, height)
        .map_err(|e| PixieError::ProcessingError(format!("JPEG encode failed: {}", e)))?;
    
    Ok(encoded)
}

/// Optimize JPEG by stripping metadata only (lossless)
fn optimize_jpeg_metadata_only(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let mut result = Vec::with_capacity(data.len());
    let mut pos = 0;
    
    while pos + 1 < data.len() {
        if data[pos] == 0xFF && pos + 1 < data.len() {
            let marker = data[pos + 1];
            
            match marker {
                0x00 | 0xFF => {
                    // Padding or escape - keep
                    result.push(data[pos]);
                    pos += 1;
                },
                0xD8 => {
                    // Start of Image - always keep
                    result.extend_from_slice(&data[pos..pos + 2]);
                    pos += 2;
                },
                0xD9 => {
                    // End of Image - keep and finish
                    result.extend_from_slice(&data[pos..pos + 2]);
                    break;
                },
                0xDA => {
                    // Start of Scan - keep everything from here
                    result.extend_from_slice(&data[pos..]);
                    break;
                },
                0xE1 => {
                    // EXIF data - strip for quality < 80
                    if let Some(segment_end) = get_segment_end(data, pos) {
                        if quality >= 80 {
                            result.extend_from_slice(&data[pos..segment_end]);
                        }
                        pos = segment_end;
                    } else {
                        break;
                    }
                },
                0xE0 | 0xE2..=0xEF => {
                    // Other APP segments - selective keeping
                    if let Some(segment_end) = get_segment_end(data, pos) {
                        let keep = match marker {
                            0xE0 => true, // JFIF - usually keep
                            _ => quality > 90, // Others - only for highest quality
                        };
                        
                        if keep {
                            result.extend_from_slice(&data[pos..segment_end]);
                        }
                        pos = segment_end;
                    } else {
                        break;
                    }
                },
                0xFE => {
                    // Comment - strip for quality < 85
                    if let Some(segment_end) = get_segment_end(data, pos) {
                        if quality >= 85 {
                            result.extend_from_slice(&data[pos..segment_end]);
                        }
                        pos = segment_end;
                    } else {
                        break;
                    }
                },
                _ => {
                    // Other segments - keep essential ones
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
    
    Ok(result)
}

/// Get the end position of a JPEG segment
fn get_segment_end(data: &[u8], start: usize) -> Option<usize> {
    if start + 4 > data.len() {
        return None;
    }
    
    let length = u16::from_be_bytes([data[start + 2], data[start + 3]]) as usize;
    let end = start + 2 + length;
    
    if end <= data.len() {
        Some(end)
    } else {
        None
    }
}

/// Fallback to basic JPEG optimization when zune-jpeg is not available
#[cfg(not(feature = "codec-jpeg"))]
pub fn optimize_jpeg_zune(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Fall back to the basic JPEG optimization from jpeg.rs
    crate::image::jpeg::optimize_jpeg(data, quality, config)
}
