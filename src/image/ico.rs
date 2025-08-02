//! ICO (Windows Icon) optimization
//! Optimize ICO files by processing embedded images

extern crate alloc;
use alloc::{vec::Vec, string::ToString};
use crate::types::{PixieResult, ImageOptConfig, PixieError};

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

/// Optimize ICO file by optimizing each embedded image
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
    
    // For lossless mode, preserve original structure
    if config.lossless && quality > 90 {
        return Ok(data.to_vec());
    }
    
    // ICO optimization requires complex parsing of each embedded image
    // For now, we'll do basic validation and size reduction
    optimize_ico_basic(data, quality, icon_count)
}

/// Basic ICO optimization
fn optimize_ico_basic(data: &[u8], _quality: u8, icon_count: usize) -> PixieResult<Vec<u8>> {
    // Each directory entry is 16 bytes
    let dir_size = 6 + (icon_count * 16);
    
    if data.len() < dir_size {
        return Err(PixieError::InvalidFormat("ICO directory truncated".to_string()));
    }
    
    // Parse directory entries to understand structure
    let mut entries = Vec::with_capacity(icon_count);
    
    for i in 0..icon_count {
        let entry_offset = 6 + (i * 16);
        if entry_offset + 16 > data.len() {
            break;
        }
        
        let entry = &data[entry_offset..entry_offset + 16];
        
        // Parse entry structure
        let width = if entry[0] == 0 { 256 } else { entry[0] as u32 };
        let height = if entry[1] == 0 { 256 } else { entry[1] as u32 };
        let color_count = entry[2];
        let _reserved = entry[3];
        let planes = u16::from_le_bytes([entry[4], entry[5]]);
        let bit_count = u16::from_le_bytes([entry[6], entry[7]]);
        let image_size = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]);
        let image_offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]);
        
        entries.push(IconEntry {
            width,
            height,
            color_count,
            planes,
            bit_count,
            image_size,
            image_offset,
        });
    }
    
    // For now, return original data
    // Future optimization would:
    // 1. Extract each embedded image (PNG/BMP)
    // 2. Optimize each image individually
    // 3. Rebuild ICO structure
    // 4. Remove duplicate/redundant sizes
    
    Ok(data.to_vec())
}

/// ICO directory entry structure
#[derive(Debug)]
struct IconEntry {
    width: u32,
    height: u32,
    color_count: u8,
    planes: u16,
    bit_count: u16,
    image_size: u32,
    image_offset: u32,
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