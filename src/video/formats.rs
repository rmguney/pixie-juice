use std::path::Path;
use crate::common::ProcessingResult;

/// Supported video formats
#[derive(Debug, Clone, PartialEq)]
pub enum VideoFormat {
    MP4,
    WebM,
}

impl VideoFormat {
    /// Detect video format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp4" | "m4v" => Some(VideoFormat::MP4),
            "webm" => Some(VideoFormat::WebM),
            _ => None,
        }
    }
    
    /// Detect video format from file path
    pub fn from_path<P: AsRef<Path>>(path: P) -> ProcessingResult<Self> {
        let path = path.as_ref();
        let ext = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| format!("Could not determine file extension for: {}", path.display()))?;
            
        Self::from_extension(ext)
            .ok_or_else(|| format!("Unsupported video format: .{}", ext).into())
    }
    
    /// Get the typical file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::MP4 => "mp4",
            VideoFormat::WebM => "webm",
        }
    }
    
    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoFormat::MP4 => "video/mp4",
            VideoFormat::WebM => "video/webm",
        }
    }
}

/// Get all supported video file extensions
pub fn get_supported_extensions() -> Vec<&'static str> {
    vec!["mp4", "m4v", "webm"]
}

/// Check if a file extension is supported
pub fn is_supported_extension(ext: &str) -> bool {
    VideoFormat::from_extension(ext).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_detection() {
        assert_eq!(VideoFormat::from_extension("mp4"), Some(VideoFormat::MP4));
        assert_eq!(VideoFormat::from_extension("MP4"), Some(VideoFormat::MP4));
        assert_eq!(VideoFormat::from_extension("m4v"), Some(VideoFormat::MP4));
        assert_eq!(VideoFormat::from_extension("webm"), Some(VideoFormat::WebM));
        assert_eq!(VideoFormat::from_extension("WEBM"), Some(VideoFormat::WebM));
    }
    
    #[test]
    fn test_extension_support() {
        assert!(is_supported_extension("mp4"));
        assert!(is_supported_extension("webm"));
    }
}
