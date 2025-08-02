//! Image format detection and utilities

extern crate alloc;
use alloc::string::ToString;

use crate::types::{OptError, OptResult};

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    // Phase 1: Core Image Formats
    PNG,
    JPEG,
    WebP,
    GIF,
    BMP,
    // TIFF,  // Not implemented yet
    // SVG,   // Commented out - needs usvg dependency
    ICO,
    // Phase 3: Advanced Image Formats (commented out for now)
    // AVIF,
    // HEIC,
    // PDF,
    // HDR,
    // EXR,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::PNG),
            "jpg" | "jpeg" => Some(Self::JPEG),
            "webp" => Some(Self::WebP),
            "gif" => Some(Self::GIF),
            "bmp" => Some(Self::BMP),
            // "svg" => Some(Self::SVG),  // Commented out
            "ico" => Some(Self::ICO),
            // Not implemented yet
            // "tiff" | "tif" => Some(Self::TIFF),
            // Phase 3: Advanced Image Formats (commented out for now)
            // "avif" => Some(Self::AVIF),
            // "heic" | "heif" => Some(Self::HEIC),
            // "pdf" => Some(Self::PDF),
            // "hdr" => Some(Self::HDR),
            // "exr" => Some(Self::EXR),
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
            // Self::SVG => "image/svg+xml",  // Commented out
            Self::ICO => "image/vnd.microsoft.icon",
            // Not implemented yet
            // Self::TIFF => "image/tiff",
            // Phase 3: Advanced Image Formats (commented out for now)
            // Self::AVIF => "image/avif",
            // Self::HEIC => "image/heic",
            // Self::PDF => "application/pdf",
            // Self::HDR => "image/vnd.radiance",
            // Self::EXR => "image/x-exr",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::PNG => "png",
            Self::JPEG => "jpg",
            Self::WebP => "webp",
            Self::GIF => "gif",
            Self::BMP => "bmp",
            // Self::SVG => "svg",  // Commented out
            Self::ICO => "ico",
            // Not implemented yet
            // Self::TIFF => "tiff",
            // Phase 3: Advanced Image Formats (commented out for now)
            // Self::AVIF => "avif",
            // Self::HEIC => "heic",
            // Self::PDF => "pdf",
            // Self::HDR => "hdr",
            // Self::EXR => "exr",
        }
    }
}

/// Detect image format from file header (magic bytes)
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

    /*
    // TIFF: II or MM (not implemented yet)
    if data.starts_with(&[0x49, 0x49, 0x2A, 0x00]) || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A]) {
        return Ok(ImageFormat::TIFF);
    }
    */

    // Phase 3: Advanced Image Formats (commented out for now)
    /*
    // AVIF: Check for AVIF signature ftypavif
    if data.len() >= 12 && data[4..8] == [0x66, 0x74, 0x79, 0x70] && data[8..12] == [0x61, 0x76, 0x69, 0x66] {
        return Ok(ImageFormat::AVIF);
    }

    // HEIC: Check for HEIC signature ftypheic or ftypmif1
    if data.len() >= 12 && data[4..8] == [0x66, 0x74, 0x79, 0x70] {
        if data[8..12] == [0x68, 0x65, 0x69, 0x63] || // heic
           data[8..12] == [0x6d, 0x69, 0x66, 0x31] || // mif1
           data[8..12] == [0x68, 0x65, 0x69, 0x78] {   // heix
            return Ok(ImageFormat::HEIC);
        }
    }
    */

    // SVG: Check for SVG text signature (commented out)
    // if let Ok(text) = core::str::from_utf8(data) {
    //     let trimmed = text.trim_start();
    //     if trimmed.starts_with("<?xml") && trimmed.contains("<svg") ||
    //        trimmed.starts_with("<svg") {
    //         return Ok(ImageFormat::SVG);
    //     }
    // }

    /*
    // PDF: %PDF-
    if data.starts_with(b"%PDF-") {
        return Ok(ImageFormat::PDF);
    }
    */

    // ICO: 00 00 01 00
    if data.len() >= 4 && data[0] == 0x00 && data[1] == 0x00 && 
       data[2] == 0x01 && data[3] == 0x00 {
        return Ok(ImageFormat::ICO);
    }

    /*
    // HDR: #?RADIANCE or #?RGBE
    if let Ok(text) = core::str::from_utf8(data) {
        if text.starts_with("#?RADIANCE") || text.starts_with("#?RGBE") {
            return Ok(ImageFormat::HDR);
        }
    }

    // EXR: Magic number 0x762f3101
    if data.len() >= 4 && data[0] == 0x76 && data[1] == 0x2f && 
       data[2] == 0x31 && data[3] == 0x01 {
        return Ok(ImageFormat::EXR);
    }
    */

    Err(OptError::InvalidFormat("Unknown image format".to_string()))
}

/// Get basic image information without full decode
/// This is a placeholder implementation for WASM compatibility
pub fn get_image_info(data: &[u8]) -> OptResult<()> {
    // For now, just validate that it's a supported format
    let _format = detect_image_format(data)?;
    
    // TODO: Implement basic header parsing for each format
    // without requiring std::io::Cursor
    Ok(())
}
