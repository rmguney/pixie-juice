//! BMP optimization using image crate for format conversion
//! 
//! BMP files are typically uncompressed, so we convert them to PNG
//! for significant size reduction while maintaining quality.

use crate::types::{OptConfig, OptError, OptResult};

/// Optimize BMP by converting to PNG format for better compression
pub fn optimize_bmp(data: &[u8], config: &OptConfig) -> OptResult<Vec<u8>> {
    // Load BMP using image crate
    let img = image::load_from_memory(data)
        .map_err(|e| OptError::ProcessingError(format!("Failed to decode BMP: {}", e)))?;
    
    // Apply resizing if requested
    let img = if let (Some(max_width), Some(max_height)) = (config.max_width, config.max_height) {
        let (width, height) = img.dimensions();
        if width > max_width || height > max_height {
            img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        }
    } else {
        img
    };
    
    // Convert to PNG for optimization
    let mut png_data = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| OptError::ProcessingError(format!("Failed to encode PNG: {}", e)))?;
    
    // Use PNG optimization on the converted data
    super::png::optimize_png(&png_data, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bmp_optimization() {
        // This would need a real BMP file for testing
        // For now, just test that the function exists and has the right signature
        let config = OptConfig::default();
        
        // Test with empty data should fail gracefully
        let result = optimize_bmp(&[], &config);
        assert!(result.is_err());
    }
}
