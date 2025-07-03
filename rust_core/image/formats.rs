//! Image format detection and utilities

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
        }
    }
}

/// Detect image format from file header (magic bytes)
pub fn detect_image_format(data: &[u8]) -> OptResult<ImageFormat> {
    if data.len() < 12 {
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

    // TIFF: II or MM
    if data.starts_with(&[0x49, 0x49, 0x2A, 0x00]) || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A]) {
        return Ok(ImageFormat::TIFF);
    }

    Err(OptError::InvalidFormat("Unknown image format".to_string()))
}

/// Get basic image information without full decode
pub fn get_image_info(data: &[u8]) -> OptResult<super::ImageInfo> {
    use image::io::Reader as ImageReader;
    use std::io::Cursor;

    let format = detect_image_format(data)?;
    
    let mut reader = ImageReader::new(Cursor::new(data));
    reader.set_format(match format {
        ImageFormat::PNG => image::ImageFormat::Png,
        ImageFormat::JPEG => image::ImageFormat::Jpeg,
        ImageFormat::WebP => image::ImageFormat::WebP,
        ImageFormat::GIF => image::ImageFormat::Gif,
        ImageFormat::BMP => image::ImageFormat::Bmp,
        ImageFormat::TIFF => image::ImageFormat::Tiff,
    });

    let (width, height) = reader.into_dimensions()
        .map_err(|e| OptError::ProcessingError(e.to_string()))?;

    // Basic channel/bit depth estimation based on format
    let (channels, bit_depth) = match format {
        ImageFormat::PNG => (4, 8), // Assume RGBA, 8-bit (will be refined later)
        ImageFormat::JPEG => (3, 8), // RGB, 8-bit
        ImageFormat::WebP => (4, 8), // RGBA, 8-bit
        ImageFormat::GIF => (3, 8),  // RGB with palette
        ImageFormat::BMP => (3, 8),  // RGB, 8-bit
        ImageFormat::TIFF => (3, 8), // RGB, 8-bit (variable in practice)
    };

    Ok(super::ImageInfo {
        format,
        width,
        height,
        channels,
        bit_depth,
        file_size: data.len(),
    })
}
