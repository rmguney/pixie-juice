use std::path::Path;
use crate::common::{OptimizationOptions, ProcessingResult};

pub mod formats;
pub mod mp4;
pub mod webm;

pub use formats::VideoFormat;

/// Main video optimization function that routes to format-specific implementations
pub fn optimize_video<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    options: &OptimizationOptions,
) -> ProcessingResult<()> {
    let input_path = input_path.as_ref();
    let format = VideoFormat::from_path(input_path)?;
    
    match format {
        VideoFormat::MP4 => mp4::optimize(input_path, output_path.as_ref(), options),
        VideoFormat::WebM => webm::optimize(input_path, output_path.as_ref(), options),
    }
}

/// Get supported video format extensions
pub fn get_supported_extensions() -> Vec<&'static str> {
    formats::get_supported_extensions()
}

/// Check if a file extension is supported for video processing
pub fn is_supported_extension(ext: &str) -> bool {
    formats::is_supported_extension(ext)
}
