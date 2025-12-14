extern crate alloc;
use alloc::{vec::Vec, string::ToString};
use crate::types::{PixieResult, ImageOptConfig, PixieError};
use crate::optimizers::{get_current_time_ms, update_performance_stats};

pub fn is_ico(data: &[u8]) -> bool {
    if data.len() < 6 {
        return false;
    }
    data[0] == 0x00 && data[1] == 0x00 && data[2] == 0x01 && data[3] == 0x00
}

pub fn optimize_ico(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    if !is_ico(data) {
        return Err(PixieError::InvalidFormat("Not a valid ICO file".to_string()));
    }
    
    if data.len() < 6 {
        return Err(PixieError::InvalidFormat("ICO file too small".to_string()));
    }
    
    let icon_count = u16::from_le_bytes([data[4], data[5]]) as usize;
    
    if icon_count == 0 {
        return Err(PixieError::InvalidFormat("ICO file contains no icons".to_string()));
    }
    
    if config.lossless && quality > 90 {
        return optimize_ico_conservative(data);
    }
    
    let preprocessed = apply_ico_optimization(data, quality)?;
    
    let strategies = get_ico_optimization_strategies(quality, icon_count);
    let mut best_result = preprocessed;
    let mut best_size = best_result.len();
    
    for strategy in strategies {
        if let Ok(optimized) = apply_ico_strategy(&best_result, strategy, quality, config) {
            if optimized.len() < best_size {
                best_result = optimized;
                best_size = best_result.len();
            }
        }
    }
    
    if best_result.len() < data.len() {
        Ok(best_result)
    } else {
        Ok(data.to_vec())
    }
}

#[derive(Debug, Clone)]
enum IcoOptimizationStrategy {
    OptimizeEmbeddedImages,
    RemoveRedundantSizes,
    ModernRecompression,
    StripEmbeddedMetadata,
}

fn apply_ico_optimization(data: &[u8], quality: u8) -> PixieResult<Vec<u8>> {
    let start_time = get_current_time_ms();
    let data_size = data.len();
    
    let result = if quality <= 80 {
        strip_embedded_ico_metadata(data)
    } else {
        optimize_ico_conservative(data)
    };
    
    let elapsed = get_current_time_ms() - start_time;
    update_performance_stats(true, elapsed, data_size);
    
    result
}

fn optimize_ico_conservative(data: &[u8]) -> PixieResult<Vec<u8>> {
    apply_ico_strategy(data, IcoOptimizationStrategy::StripEmbeddedMetadata, 100, &ImageOptConfig::default())
}

fn get_ico_optimization_strategies(quality: u8, icon_count: usize) -> Vec<IcoOptimizationStrategy> {
    let mut strategies = Vec::new();
    strategies.push(IcoOptimizationStrategy::StripEmbeddedMetadata);
    strategies.push(IcoOptimizationStrategy::OptimizeEmbeddedImages);
    if quality <= 60 && icon_count > 3 {
        strategies.push(IcoOptimizationStrategy::RemoveRedundantSizes);
    }
    if quality <= 40 {
        strategies.push(IcoOptimizationStrategy::ModernRecompression);
    }
    
    strategies
}

fn apply_ico_strategy(data: &[u8], strategy: IcoOptimizationStrategy, quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    match strategy {
        IcoOptimizationStrategy::OptimizeEmbeddedImages => optimize_embedded_ico_images(data, quality, config),
        IcoOptimizationStrategy::RemoveRedundantSizes => remove_redundant_ico_sizes(data),
        IcoOptimizationStrategy::ModernRecompression => modern_ico_recompression(data, quality),
        IcoOptimizationStrategy::StripEmbeddedMetadata => strip_embedded_ico_metadata(data),
    }
}

fn optimize_embedded_ico_images(data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    Ok(data.to_vec())
}

fn remove_redundant_ico_sizes(data: &[u8]) -> PixieResult<Vec<u8>> {
    Ok(data.to_vec())
}

fn modern_ico_recompression(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    Ok(data.to_vec())
}

fn strip_embedded_ico_metadata(data: &[u8]) -> PixieResult<Vec<u8>> {
    Ok(data.to_vec())
}

pub fn get_ico_info(data: &[u8]) -> PixieResult<(u32, u32, u8)> {
    if !is_ico(data) {
        return Err(PixieError::InvalidFormat("Not a valid ICO file".to_string()));
    }
    
    if data.len() < 22 {
        return Err(PixieError::InvalidFormat("ICO file too small".to_string()));
    }
    
    let icon_count = u16::from_le_bytes([data[4], data[5]]);
    
    if icon_count == 0 {
        return Err(PixieError::InvalidFormat("ICO file contains no icons".to_string()));
    }
    
    let first_entry = &data[6..22];
    
    let width = if first_entry[0] == 0 { 256 } else { first_entry[0] as u32 };
    let height = if first_entry[1] == 0 { 256 } else { first_entry[1] as u32 };
    let bit_count = u16::from_le_bytes([first_entry[6], first_entry[7]]);
    
    let bits_per_pixel = match bit_count {
        1 | 4 | 8 => 8,
        16 => 16,
        24 => 24,
        32 => 32,
        _ => 32, // Default to 32-bit
    };
    
    Ok((width, height, bits_per_pixel as u8))
}

pub fn parse_ico_dimensions(data: &[u8]) -> PixieResult<(u32, u32)> {
    let (width, height, _) = get_ico_info(data)?;
    Ok((width, height))
}

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
        let ico_header = [0x00, 0x00, 0x01, 0x00, 0x01, 0x00];
        assert!(is_ico(&ico_header));
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