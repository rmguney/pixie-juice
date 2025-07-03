//! Video format handling

use crate::{OptError, OptResult};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoFormat {
    Mp4,
    WebM,
    Avi,
    Mov,
}

impl VideoFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> OptResult<Self> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| OptError::FormatError("No file extension".to_string()))?;
        
        match ext.to_lowercase().as_str() {
            "mp4" => Ok(Self::Mp4),
            "webm" => Ok(Self::WebM),
            "avi" => Ok(Self::Avi),
            "mov" => Ok(Self::Mov),
            _ => Err(OptError::FormatError(format!("Unsupported video format: {}", ext))),
        }
    }
    
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::WebM => "webm",
            Self::Avi => "avi",
            Self::Mov => "mov",
        }
    }
    
    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Mp4 => "video/mp4",
            Self::WebM => "video/webm",
            Self::Avi => "video/x-msvideo",
            Self::Mov => "video/quicktime",
        }
    }
    
    /// Check if format is web-compatible
    pub fn web_compatible(&self) -> bool {
        matches!(self, Self::Mp4 | Self::WebM)
    }
    
    /// Check if format supports streaming
    pub fn supports_streaming(&self) -> bool {
        matches!(self, Self::Mp4 | Self::WebM)
    }
}
