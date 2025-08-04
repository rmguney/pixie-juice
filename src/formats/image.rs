//! Image format handling

extern crate alloc;
use alloc::{format, string::ToString};
use crate::{OptError, OptResult};
// WASM-compatible format detection - no std::path usage

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Gif,
    Bmp,
    Tiff,
    Ico,
    Svg,
    Tga,
    Avif,
}

impl ImageFormat {
    /// Detect format from file extension
    pub fn from_extension(filename: &str) -> OptResult<Self> {
        let ext = filename
            .split('.')
            .last()
            .ok_or_else(|| OptError::FormatError("No file extension".to_string()))?;
        
        match ext.to_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "webp" => Ok(Self::WebP),
            "gif" => Ok(Self::Gif),
            "bmp" => Ok(Self::Bmp),
            "tiff" | "tif" => Ok(Self::Tiff),
            "ico" => Ok(Self::Ico),
            "svg" => Ok(Self::Svg),
            "tga" | "targa" => Ok(Self::Tga),
            "avif" => Ok(Self::Avif),
            _ => Err(OptError::FormatError(format!("Unsupported image format: {}", ext))),
        }
    }
    
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Ico => "ico",
            Self::Svg => "svg",
            Self::Tga => "tga",
            Self::Avif => "avif",
        }
    }
    
    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::WebP => "image/webp",
            Self::Gif => "image/gif",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Ico => "image/x-icon",
            Self::Svg => "image/svg+xml",
            Self::Tga => "image/x-tga",
            Self::Avif => "image/avif",
        }
    }
    
    /// Check if format supports lossless compression
    pub fn supports_lossless(&self) -> bool {
        matches!(self, Self::Png | Self::WebP | Self::Gif | Self::Bmp | Self::Tiff | Self::Tga | Self::Avif)
    }
    
    /// Check if format supports quality settings
    pub fn supports_quality(&self) -> bool {
        matches!(self, Self::Jpeg | Self::WebP | Self::Avif)
    }
    
    /// Detect format from file header (magic bytes)
    pub fn from_header(data: &[u8]) -> OptResult<Self> {
        if data.len() < 8 {
            return Err(OptError::FormatError("Insufficient data for format detection".to_string()));
        }
        
        // PNG: 89 50 4E 47 0D 0A 1A 0A
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Ok(Self::Png);
        }
        
        // JPEG: FF D8 FF
        if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            return Ok(Self::Jpeg);
        }
        
        // WebP: "RIFF" ... "WEBP"
        if data.len() >= 12 && 
           &data[0..4] == b"RIFF" && 
           &data[8..12] == b"WEBP" {
            return Ok(Self::WebP);
        }
        
        // GIF: "GIF87a" or "GIF89a"
        if data.len() >= 6 && 
           (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
            return Ok(Self::Gif);
        }
        
        // BMP: "BM"
        if data.len() >= 2 && &data[0..2] == b"BM" {
            return Ok(Self::Bmp);
        }
        
        // TIFF: "II" (little-endian) or "MM" (big-endian) followed by 42
        if data.len() >= 4 &&
           ((&data[0..2] == b"II" && data[2] == 42 && data[3] == 0) ||
            (&data[0..2] == b"MM" && data[2] == 0 && data[3] == 42)) {
            return Ok(Self::Tiff);
        }
        
        // SVG: Check for SVG text signatures
        if let Ok(text) = core::str::from_utf8(data) {
            let trimmed = text.trim_start();
            // Check for common SVG signatures
            if (trimmed.starts_with("<?xml") && text.contains("<svg")) ||
               trimmed.starts_with("<svg") ||
               (trimmed.starts_with("<!DOCTYPE") && text.contains("<svg")) {
                return Ok(Self::Svg);
            }
        }
        
        // ICO: First 4 bytes are [0, 0, 1, 0] for ICO format
        if data.len() >= 4 && data[0] == 0 && data[1] == 0 && data[2] == 1 && data[3] == 0 {
            return Ok(Self::Ico);
        }
        
        // TGA: Has no magic signature, use footer or heuristics
        // TGA v2.0 has a footer "TRUEVISION-XFILE." at the end
        if data.len() >= 26 {
            let footer_start = data.len() - 26;
            if &data[footer_start..footer_start + 16] == b"TRUEVISION-XFILE" {
                return Ok(Self::Tga);
            }
        }
        // TGA v1.0 heuristic: check if first byte is valid image type (0-11)
        if data.len() >= 18 && data[2] <= 11 {
            // This is a weak heuristic, but TGA has no magic bytes
            // We'll validate more thoroughly in the TGA loader
            let width = u16::from_le_bytes([data[12], data[13]]);
            let height = u16::from_le_bytes([data[14], data[15]]);
            let bpp = data[16];
            if width > 0 && height > 0 && (bpp == 8 || bpp == 16 || bpp == 24 || bpp == 32) {
                return Ok(Self::Tga);
            }
        }
        
        // AVIF: Check for AVIF signature ftypavif  
        if data.len() >= 12 && 
           &data[4..8] == b"ftyp" && 
           &data[8..12] == b"avif" {
            return Ok(Self::Avif);
        }
        
        Err(OptError::FormatError("Unknown image format".to_string()))
    }
}

/// Detect image format from file header (magic bytes) - public function  
pub fn detect_image_format(data: &[u8]) -> OptResult<ImageFormat> {
    ImageFormat::from_header(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("test.png").unwrap(), ImageFormat::Png);
        assert_eq!(ImageFormat::from_extension("test.jpg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_extension("test.jpeg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_extension("test.webp").unwrap(), ImageFormat::WebP);
        
        assert!(ImageFormat::from_extension("test.unknown").is_err());
    }
    
    #[test]
    fn test_format_properties() {
        assert!(ImageFormat::Png.supports_lossless());
        assert!(!ImageFormat::Png.supports_quality());
        
        assert!(!ImageFormat::Jpeg.supports_lossless());
        assert!(ImageFormat::Jpeg.supports_quality());
        
        assert!(ImageFormat::WebP.supports_lossless());
        assert!(ImageFormat::WebP.supports_quality());
    }
    
    #[test]
    fn test_format_from_header() {
        // PNG header
        let png_header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(ImageFormat::from_header(&png_header).unwrap(), ImageFormat::Png);
        
        // JPEG header
        let jpeg_header = [0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(ImageFormat::from_header(&jpeg_header).unwrap(), ImageFormat::Jpeg);
        
        // Unknown header
        let unknown_header = [0x00, 0x01, 0x02, 0x03];
        assert!(ImageFormat::from_header(&unknown_header).is_err());
    }
}
