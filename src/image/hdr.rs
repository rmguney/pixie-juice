//! HDR/EXR (High Dynamic Range) image optimization
//! Optimize HDR and OpenEXR format images

extern crate alloc;
use alloc::{vec::Vec, string::ToString};
use crate::types::{PixieResult, ImageOptConfig, PixieError};

/// Check if data is HDR format (Radiance HDR)
pub fn is_hdr(data: &[u8]) -> bool {
    if data.len() < 10 {
        return false;
    }
    
    // Check for Radiance HDR signature
    let text = core::str::from_utf8(data).unwrap_or("");
    text.starts_with("#?RADIANCE") || text.starts_with("#?RGBE")
}

/// Check if data is EXR format
pub fn is_exr(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    
    // EXR magic number: 0x762f3101
    data[0] == 0x76 && data[1] == 0x2f && data[2] == 0x31 && data[3] == 0x01
}

/// Optimize HDR image
#[cfg(feature = "codec-hdr")]
pub fn optimize_hdr(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate HDR format
    if !is_hdr(data) {
        return Err(PixieError::InvalidFormat("Not a valid HDR file".to_string()));
    }
    
    // For lossless mode with high quality, preserve original
    if config.lossless && quality > 90 {
        return Ok(data.to_vec());
    }
    
    // Note: Advanced HDR optimization would require exr crate implementation
    // For now, use basic optimization
    optimize_hdr_basic(data, quality)
}

/// Fallback for when HDR codec features are not available
#[cfg(not(feature = "codec-hdr"))]
pub fn optimize_hdr(data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate HDR format
    if !is_hdr(data) {
        return Err(PixieError::InvalidFormat("Not a valid HDR file".to_string()));
    }
    
    // Return original data - HDR optimization requires specialized handling
    Ok(data.to_vec())
}

/// Optimize EXR image
#[cfg(feature = "codec-hdr")]
pub fn optimize_exr(data: &[u8], quality: u8, config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate EXR format
    if !is_exr(data) {
        return Err(PixieError::InvalidFormat("Not a valid EXR file".to_string()));
    }
    
    // For lossless mode with high quality, preserve original
    if config.lossless && quality > 90 {
        return Ok(data.to_vec());
    }
    
    // Note: Advanced EXR optimization would require exr crate implementation
    // For now, use basic optimization
    optimize_exr_basic(data, quality)
}

/// Fallback for when HDR codec features are not available
#[cfg(not(feature = "codec-hdr"))]
pub fn optimize_exr(data: &[u8], _quality: u8, _config: &ImageOptConfig) -> PixieResult<Vec<u8>> {
    // Validate EXR format
    if !is_exr(data) {
        return Err(PixieError::InvalidFormat("Not a valid EXR file".to_string()));
    }
    
    // Return original data - EXR optimization requires specialized handling
    Ok(data.to_vec())
}

/// Basic HDR optimization without external crates
fn optimize_hdr_basic(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    // HDR files have a text header followed by pixel data
    let text = core::str::from_utf8(data).unwrap_or("");
    
    // Find the end of header (empty line followed by resolution line)
    let lines: Vec<&str> = text.lines().collect();
    let mut header_end = 0;
    let mut found_format = false;
    
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("FORMAT=") {
            found_format = true;
        }
        if found_format && line.trim().is_empty() {
            // Next line should be resolution
            if i + 1 < lines.len() && lines[i + 1].contains("X") && lines[i + 1].contains("Y") {
                // Calculate byte offset to end of resolution line
                let mut byte_offset = 0;
                for j in 0..=i+1 {
                    byte_offset += lines[j].len() + 1; // +1 for newline
                }
                header_end = byte_offset;
                break;
            }
        }
    }
    
    if header_end == 0 {
        // Could not parse header, return original
        return Ok(data.to_vec());
    }
    
    // For basic optimization, we can only preserve the structure
    // Advanced optimization would require understanding RGBE encoding
    Ok(data.to_vec())
}

/// Basic EXR optimization without external crates
fn optimize_exr_basic(data: &[u8], _quality: u8) -> PixieResult<Vec<u8>> {
    // EXR has complex binary structure
    // Without proper parsing, we can only validate and preserve
    
    if data.len() < 20 {
        return Err(PixieError::InvalidFormat("EXR file too small".to_string()));
    }
    
    // Basic validation of EXR structure
    // Version should be 2 (bytes 4-7)
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != 2 {
        return Err(PixieError::InvalidFormat("Unsupported EXR version".to_string()));
    }
    
    // For basic optimization, return original
    // Advanced optimization would require:
    // 1. Parse attribute headers
    // 2. Identify compression methods
    // 3. Re-compress with better settings
    Ok(data.to_vec())
}

/// Get HDR metadata
pub fn get_hdr_info(data: &[u8]) -> PixieResult<(u32, u32, u8)> {
    if !is_hdr(data) {
        return Err(PixieError::InvalidFormat("Not a valid HDR file".to_string()));
    }
    
    let text = core::str::from_utf8(data).unwrap_or("");
    let lines: Vec<&str> = text.lines().collect();
    
    // Look for resolution line (format: -Y height +X width)
    for line in lines {
        if line.contains("-Y") && line.contains("+X") {
            // Parse resolution line: "-Y 512 +X 512"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let (Ok(height), Ok(width)) = (parts[1].parse::<u32>(), parts[3].parse::<u32>()) {
                    return Ok((width, height, 32)); // HDR is typically 32-bit float per channel
                }
            }
        }
    }
    
    // Default dimensions if not found
    Ok((512, 512, 32))
}

/// Get EXR metadata
pub fn get_exr_info(data: &[u8]) -> PixieResult<(u32, u32, u8)> {
    if !is_exr(data) {
        return Err(PixieError::InvalidFormat("Not a valid EXR file".to_string()));
    }
    
    if data.len() < 20 {
        return Err(PixieError::InvalidFormat("EXR file too small".to_string()));
    }
    
    // Parse EXR attributes to find dataWindow
    let mut offset = 8; // Skip magic and version
    let mut width = 512u32;
    let mut height = 512u32;
    
    // Parse attribute headers
    while offset + 5 < data.len() {
        // Read attribute name length
        let name_len = data[offset] as usize;
        if name_len == 0 {
            break; // End of attributes
        }
        
        offset += 1;
        if offset + name_len > data.len() {
            break;
        }
        
        let name = &data[offset..offset + name_len];
        offset += name_len;
        
        if offset + 4 > data.len() {
            break;
        }
        
        // Read type name length
        let type_len = data[offset] as usize;
        offset += 1;
        
        if offset + type_len > data.len() {
            break;
        }
        
        offset += type_len; // Skip type name
        
        if offset + 4 > data.len() {
            break;
        }
        
        // Read attribute size
        let attr_size = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
        offset += 4;
        
        // Check if this is dataWindow attribute
        if name == b"dataWindow" && attr_size >= 16 {
            if offset + 16 <= data.len() {
                let xmin = i32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
                let ymin = i32::from_le_bytes([data[offset+4], data[offset+5], data[offset+6], data[offset+7]]);
                let xmax = i32::from_le_bytes([data[offset+8], data[offset+9], data[offset+10], data[offset+11]]);
                let ymax = i32::from_le_bytes([data[offset+12], data[offset+13], data[offset+14], data[offset+15]]);
                
                width = (xmax - xmin + 1) as u32;
                height = (ymax - ymin + 1) as u32;
                break;
            }
        }
        
        offset += attr_size;
    }
    
    Ok((width, height, 32)) // EXR is typically 32-bit float per channel
}

/// Parse HDR dimensions
pub fn parse_hdr_dimensions(data: &[u8]) -> PixieResult<(u32, u32)> {
    let (width, height, _) = get_hdr_info(data)?;
    Ok((width, height))
}

/// Parse EXR dimensions
pub fn parse_exr_dimensions(data: &[u8]) -> PixieResult<(u32, u32)> {
    let (width, height, _) = get_exr_info(data)?;
    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hdr_detection() {
        let hdr_header = b"#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n";
        assert!(is_hdr(hdr_header));
        
        let rgbe_header = b"#?RGBE\n";
        assert!(is_hdr(rgbe_header));
        
        let not_hdr = b"\x89PNG\r\n\x1a\n";
        assert!(!is_hdr(not_hdr));
    }
    
    #[test]
    fn test_exr_detection() {
        let exr_header = [0x76, 0x2f, 0x31, 0x01];
        assert!(is_exr(&exr_header));
        
        let not_exr = b"\x89PNG\r\n\x1a\n";
        assert!(!is_exr(not_exr));
    }
}
