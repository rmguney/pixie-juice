//! ICO format support

extern crate alloc;
use alloc::{vec::Vec, string::ToString};
use crate::types::{PixieResult, ImageOptConfig, PixieError};
use crate::optimizers::{get_current_time_ms, update_performance_stats};

/// Check if data is ICO format
pub fn is_ico(data: &[u8]) -> bool {
    if data.len() < 6 {
        return false;
    }
    
    // ICO files start with reserved field (2 bytes: 0x00 0x00)
    // followed by type field (2 bytes: 0x01 0x00 for ICO)
    // followed by count field (2 bytes)
    data[0] == 0x00 && data[1] == 0x00 && data[2] == 0x01 && data[3] == 0x00
}

/// Optimize ICO file with multi-resolution optimization
pub fn optimize_ico(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate ICO format
    if !is_ico(data) {
        return Err(PixieError::InvalidFormat("Not a valid ICO file".to_string()));
    }
    
    if data.len() < 6 {
        return Err(PixieError::InvalidFormat("ICO file too small".to_string()));
    }
    
    // Parse ICO header
    let icon_count = u16::from_le_bytes([data[4], data[5]]) as usize;
    
    if icon_count == 0 {
        return Err(PixieError::InvalidFormat("ICO file contains no icons".to_string()));
    }
    
    // For lossless mode, only apply conservative optimizations
    if config.lossless && quality > 90 {
        return optimize_ico_conservative(data);
    }
    
    // Apply ICO optimization using proven libraries
    let preprocessed = apply_ico_optimization(data, quality)?;
    
    // Strategy selection based on quality
    let strategies = get_ico_optimization_strategies(quality, icon_count);
    let mut best_result = preprocessed;
    let mut best_size = best_result.len();
    
    // Try each optimization strategy and keep the best result
    for strategy in strategies {
        if let Ok(optimized) = apply_ico_strategy(&best_result, strategy, quality, config) {
            if optimized.len() < best_size {
                best_result = optimized;
                best_size = best_result.len();
            }
        }
    }
    
    // Return optimized version if smaller, otherwise original
    if best_result.len() < data.len() {
        Ok(best_result)
    } else {
        Ok(data.to_vec())
    }
}

/// ICO optimization strategies
#[derive(Debug, Clone)]
enum IcoOptimizationStrategy {
    /// Optimize embedded PNG/BMP images
    OptimizeEmbeddedImages,
    /// Remove redundant resolutions
    RemoveRedundantSizes,
    /// Recompress using modern formats
    ModernRecompression,
    /// Strip metadata from embedded images
    StripEmbeddedMetadata,
}

/// Apply ICO optimization using proven libraries
fn apply_ico_optimization(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let start_time = get_current_time_ms();
    let data_size = data.len();
    
    let result = if quality <= 80 {
        // Strip metadata from embedded PNG images
        strip_embedded_ico_metadata(data)
    } else {
        // Conservative optimization for high quality
        optimize_ico_conservative(data)
    };
    
    let elapsed = get_current_time_ms() - start_time;
    update_performance_stats(true, elapsed, data_size);
    
    result
}

/// Conservative ICO optimization for lossless mode
fn optimize_ico_conservative(data: &[u8]) -> PixieResult<Vec<u8>> {
    // Only strip metadata without changing image data
    apply_ico_strategy(data, IcoOptimizationStrategy::StripEmbeddedMetadata, 100, &ImageOptConfig::default())
}

/// Get ICO optimization strategies based on quality
fn get_ico_optimization_strategies(quality: u8, icon_count: usize) -> Vec<IcoOptimizationStrategy> {
    let mut strategies = Vec::new();
    
    // Always try to strip metadata from embedded images
    strategies.push(IcoOptimizationStrategy::StripEmbeddedMetadata);
    
    // Optimize embedded images
    strategies.push(IcoOptimizationStrategy::OptimizeEmbeddedImages);
    
    // Remove redundant sizes for low quality
    if quality <= 60 && icon_count > 3 {
        strategies.push(IcoOptimizationStrategy::RemoveRedundantSizes);
    }
    
    // Modern recompression for very low quality
    if quality <= 40 {
        strategies.push(IcoOptimizationStrategy::ModernRecompression);
    }
    
    strategies
}

/// Apply specific ICO optimization strategy
fn apply_ico_strategy(data: &[u8], strategy: IcoOptimizationStrategy, quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    match strategy {
        IcoOptimizationStrategy::OptimizeEmbeddedImages => optimize_embedded_ico_images(data, quality, config),
        IcoOptimizationStrategy::RemoveRedundantSizes => remove_redundant_ico_sizes(data),
        IcoOptimizationStrategy::ModernRecompression => modern_ico_recompression(data, quality),
        IcoOptimizationStrategy::StripEmbeddedMetadata => strip_embedded_ico_metadata(data),
    }
}

/// Optimize embedded images within ICO
fn optimize_embedded_ico_images(data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Parse ICO structure and optimize each embedded image
    // This is a complex operation that requires:
    // 1. Parsing ICO directory
    // 2. Extracting each embedded PNG/BMP
    // 3. Optimizing each image individually
    // 4. Rebuilding ICO structure
    
    // For now, return original data
    // Full implementation would be quite extensive
    Ok(data.to_vec())
}

/// Remove redundant ICO sizes (keep only essential sizes)
fn remove_redundant_ico_sizes(data: &[u8]) -> PixieResult<Vec<u8>> {
    // Analysis would determine which sizes to keep (e.g., 16x16, 32x32, 48x48)
    // and remove intermediate or very large sizes
    
    // For now, return original data
    Ok(data.to_vec())
}

/// Modern recompression using PNG with better settings
fn modern_ico_recompression(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    // Convert all embedded images to optimized PNG format
    
    // For now, return original data
    Ok(data.to_vec())
}

/// Strip metadata from embedded PNG/BMP images
fn strip_embedded_ico_metadata(data: &[u8]) -> PixieResult<Vec<u8>> {
    // Parse ICO and strip metadata from each embedded image
    
    // For now, return original data
    Ok(data.to_vec())
}

/// Get ICO metadata
pub fn get_ico_info(data: &[u8]) -> PixieResult<(u32, u32, u8)> {
    if !is_ico(data) {
        return Err(PixieError::InvalidFormat("Not a valid ICO file".to_string()));
    }
    
    if data.len() < 22 { // Header + at least one directory entry
        return Err(PixieError::InvalidFormat("ICO file too small".to_string()));
    }
    
    let icon_count = u16::from_le_bytes([data[4], data[5]]);
    
    if icon_count == 0 {
        return Err(PixieError::InvalidFormat("ICO file contains no icons".to_string()));
    }
    
    // Get info from first (largest) icon
    let first_entry = &data[6..22];
    
    let width = if first_entry[0] == 0 { 256 } else { first_entry[0] as u32 };
    let height = if first_entry[1] == 0 { 256 } else { first_entry[1] as u32 };
    let bit_count = u16::from_le_bytes([first_entry[6], first_entry[7]]);
    
    // Convert bit count to standardized format
    let bits_per_pixel = match bit_count {
        1 | 4 | 8 => 8,
        16 => 16,
        24 => 24,
        32 => 32,
        _ => 32, // Default to 32-bit
    };
    
    Ok((width, height, bits_per_pixel as u8))
}

/// Parse ICO dimensions (returns largest icon size)
pub fn parse_ico_dimensions(data: &[u8]) -> PixieResult<(u32, u32)> {
    let (width, height, _) = get_ico_info(data)?;
    Ok((width, height))
}

/// Count icons in ICO file
pub fn count_ico_icons(data: &[u8]) -> PixieResult<u16> {
    if !is_ico(data) {
        return Err(PixieError::InvalidFormat("Not a valid ICO file".to_string()));
    }
    
    if data.len() < 6 {
        return Err(PixieError::InvalidFormat("ICO file too small".to_string()));
    }
    
    let icon_count = u16::from_le_bytes([data[4], data[5]]);
    Ok(icon_count)
}

/// Get all icon sizes in ICO file
pub fn get_ico_sizes(data: &[u8]) -> PixieResult<Vec<(u32, u32)>> {
    if !is_ico(data) {
        return Err(PixieError::InvalidFormat("Not a valid ICO file".to_string()));
    }
    
    if data.len() < 6 {
        return Err(PixieError::InvalidFormat("ICO file too small".to_string()));
    }
    
    let icon_count = u16::from_le_bytes([data[4], data[5]]) as usize;
    let mut sizes = Vec::with_capacity(icon_count);
    
    for i in 0..icon_count {
        let entry_offset = 6 + (i * 16);
        if entry_offset + 16 > data.len() {
            break;
        }
        
        let entry = &data[entry_offset..entry_offset + 16];
        let width = if entry[0] == 0 { 256 } else { entry[0] as u32 };
        let height = if entry[1] == 0 { 256 } else { entry[1] as u32 };
        
        sizes.push((width, height));
    }
    
    Ok(sizes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ico_detection() {
        // ICO header: reserved(2) + type(2) + count(2)
        let ico_header = [0x00, 0x00, 0x01, 0x00, 0x01, 0x00];
        assert!(is_ico(&ico_header));
        
        // CUR header (cursor)
        let cur_header = [0x00, 0x00, 0x02, 0x00, 0x01, 0x00];
        assert!(!is_ico(&cur_header));
        
        let not_ico = b"\x89PNG\r\n\x1a\n";
        assert!(!is_ico(not_ico));
    }
    
    #[test]
    fn test_ico_count() {
        // ICO with 3 icons
        let ico_header = [0x00, 0x00, 0x01, 0x00, 0x03, 0x00];
        let result = count_ico_icons(&ico_header);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }
}