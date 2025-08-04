//! Image format detection and utilities

extern crate alloc;
use alloc::string::ToString;

use crate::types::{OptError, OptResult};

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    PNG,
    JPEG,
    WebP,
    GIF,
    BMP,
    TIFF,
    SVG,
    ICO,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::PNG),
            "jpg" | "jpeg" => Some(Self::JPEG),
            "webp" => Some(Self::WebP),
            "gif" => Some(Self::GIF),
            "bmp" => Some(Self::BMP),
            "tiff" | "tif" => Some(Self::TIFF),
            "svg" => Some(Self::SVG),
            "ico" => Some(Self::ICO),
            _ => None,
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::PNG => "image/png",
            Self::JPEG => "image/jpeg",
            Self::WebP => "image/webp",
            Self::GIF => "image/gif",
            Self::BMP => "image/bmp",
            Self::TIFF => "image/tiff",
            Self::SVG => "image/svg+xml",
            Self::ICO => "image/vnd.microsoft.icon",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::PNG => "png",
            Self::JPEG => "jpg",
            Self::WebP => "webp",
            Self::GIF => "gif",
            Self::BMP => "bmp",
            Self::TIFF => "tiff",
            Self::SVG => "svg",
            Self::ICO => "ico",
        }
    }
}

pub fn detect_image_format(data: &[u8]) -> OptResult<ImageFormat> {
    if data.len() < 8 {
        return Err(OptError::InvalidFormat("File too small".to_string()));
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Ok(ImageFormat::PNG);
    }

    // JPEG: FF D8 FF
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(ImageFormat::JPEG);
    }

    // WebP: RIFF....WEBP
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return Ok(ImageFormat::WebP);
    }

    // GIF: GIF87a or GIF89a
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return Ok(ImageFormat::GIF);
    }

    // BMP: BM
    if data.starts_with(b"BM") {
        return Ok(ImageFormat::BMP);
    }

    // TIFF: II (Intel/little-endian) or MM (Motorola/big-endian) 
    if data.starts_with(&[0x49, 0x49, 0x2A, 0x00]) || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A]) {
        return Ok(ImageFormat::TIFF);
    }

    // SVG: Check for SVG text signature (FIXED - now working)
    if let Ok(text) = core::str::from_utf8(data) {
        let trimmed = text.trim_start();
        if trimmed.starts_with("<?xml") && trimmed.contains("<svg") ||
           trimmed.starts_with("<svg") {
            return Ok(ImageFormat::SVG);
        }
    }

    // ICO: 00 00 01 00
    if data.len() >= 4 && data[0] == 0x00 && data[1] == 0x00 && 
       data[2] == 0x01 && data[3] == 0x00 {
        return Ok(ImageFormat::ICO);
    }

    // SVG: Check for SVG text signature (FIXED - consolidated detection)
    if let Ok(text) = core::str::from_utf8(data) {
        let trimmed = text.trim_start();
        if trimmed.starts_with("<?xml") && trimmed.contains("<svg") ||
           trimmed.starts_with("<svg") {
            return Ok(ImageFormat::SVG);
        }
    }

    Err(OptError::InvalidFormat("Unknown image format".to_string()))
}

/// Get basic image information without full decode
/// placeholder implementation for WASM compatibility
pub fn get_image_info(data: &[u8]) -> OptResult<()> {
    // For now, just validate that it's a supported format
    let _format = detect_image_format(data)?;
    
    // TODO: Implement basic header parsing for each format
    // without requiring std::io::Cursor
    Ok(())
}
