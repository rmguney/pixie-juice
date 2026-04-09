extern crate alloc;
use alloc::{vec::Vec, string::ToString};
use crate::types::{PixieResult, ImageOptConfig, PixieError};
use crate::optimizers::{get_current_time_ms, update_performance_stats};

const ICO_MAGIC: [u8; 4] = [0x00, 0x00, 0x01, 0x00];
const PNG_MAGIC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

pub fn is_ico(data: &[u8]) -> bool {
    data.len() >= 6 && data[..4] == ICO_MAGIC
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

    let start_time = get_current_time_ms();
    let data_size = data.len();

    let mut entries = parse_ico_entries(data)?;
    let original_size = serialize_ico(&entries).len();

    apply_strip_metadata(&mut entries);

    if !config.lossless {
        apply_remove_redundant_sizes(&mut entries, quality);
    }

    apply_optimize_embedded(&mut entries, quality, config)?;

    if quality <= 40 {
        apply_modern_recompression(&mut entries)?;
    }

    let result = serialize_ico(&entries);

    let elapsed = get_current_time_ms() - start_time;
    update_performance_stats(true, elapsed, data_size);

    if result.len() < original_size {
        Ok(result)
    } else {
        Ok(data.to_vec())
    }
}

#[derive(Clone)]
struct IcoEntry {
    width: u8,
    height: u8,
    color_count: u8,
    reserved: u8,
    planes: u16,
    bpp: u16,
    image_data: Vec<u8>,
}

impl IcoEntry {
    fn dimensions(&self) -> (u32, u32) {
        let w = if self.width == 0 { 256 } else { self.width as u32 };
        let h = if self.height == 0 { 256 } else { self.height as u32 };
        (w, h)
    }

    fn is_png(&self) -> bool {
        self.image_data.len() >= 8 && self.image_data[..8] == PNG_MAGIC
    }
}

fn parse_ico_entries(data: &[u8]) -> PixieResult<Vec<IcoEntry>> {
    if data.len() < 6 {
        return Err(PixieError::InvalidFormat("ICO header truncated".to_string()));
    }

    let icon_count = u16::from_le_bytes([data[4], data[5]]) as usize;
    let directory_end = 6 + icon_count * 16;
    if data.len() < directory_end {
        return Err(PixieError::InvalidFormat("ICO directory truncated".to_string()));
    }

    let mut entries = Vec::with_capacity(icon_count);
    for i in 0..icon_count {
        let entry_start = 6 + i * 16;
        let entry = &data[entry_start..entry_start + 16];

        let bytes_in_res = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
        let image_offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;

        if image_offset.saturating_add(bytes_in_res) > data.len() {
            return Err(PixieError::InvalidFormat("ICO entry references out-of-bounds data".to_string()));
        }

        entries.push(IcoEntry {
            width: entry[0],
            height: entry[1],
            color_count: entry[2],
            reserved: entry[3],
            planes: u16::from_le_bytes([entry[4], entry[5]]),
            bpp: u16::from_le_bytes([entry[6], entry[7]]),
            image_data: data[image_offset..image_offset + bytes_in_res].to_vec(),
        });
    }

    Ok(entries)
}

fn serialize_ico(entries: &[IcoEntry]) -> Vec<u8> {
    let header_size = 6 + entries.len() * 16;
    let mut total_size = header_size;
    for e in entries {
        total_size += e.image_data.len();
    }

    let mut out = Vec::with_capacity(total_size);
    out.extend_from_slice(&ICO_MAGIC);
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());

    let mut offset = header_size as u32;
    for e in entries {
        out.push(e.width);
        out.push(e.height);
        out.push(e.color_count);
        out.push(e.reserved);
        out.extend_from_slice(&e.planes.to_le_bytes());
        out.extend_from_slice(&e.bpp.to_le_bytes());
        out.extend_from_slice(&(e.image_data.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += e.image_data.len() as u32;
    }

    for e in entries {
        out.extend_from_slice(&e.image_data);
    }

    out
}

fn apply_strip_metadata(entries: &mut [IcoEntry]) {
    for e in entries.iter_mut() {
        if e.is_png() {
            if let Some(stripped) = strip_ancillary_png_chunks(&e.image_data) {
                if stripped.len() < e.image_data.len() {
                    e.image_data = stripped;
                }
            }
        }
    }
}

fn strip_ancillary_png_chunks(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 8 || data[..8] != PNG_MAGIC {
        return None;
    }

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&PNG_MAGIC);

    let mut pos = 8;
    while pos + 8 <= data.len() {
        let length = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let chunk_end = pos.checked_add(12)?.checked_add(length)?;
        if chunk_end > data.len() {
            return None;
        }

        let chunk_type = &data[pos + 4..pos + 8];
        if is_critical_png_chunk(chunk_type) {
            out.extend_from_slice(&data[pos..chunk_end]);
        }

        if chunk_type == b"IEND" {
            return Some(out);
        }

        pos = chunk_end;
    }

    None
}

fn is_critical_png_chunk(chunk_type: &[u8]) -> bool {
    matches!(
        chunk_type,
        b"IHDR" | b"PLTE" | b"IDAT" | b"IEND" | b"tRNS" | b"sRGB" | b"gAMA"
    )
}

fn apply_remove_redundant_sizes(entries: &mut Vec<IcoEntry>, quality: u8) {
    if entries.len() <= 1 {
        return;
    }

    let preserve_all_sizes = quality > 60;
    if preserve_all_sizes {
        return;
    }

    let preferred_sizes: &[(u32, u32)] = &[
        (16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256),
    ];

    let mut kept: Vec<IcoEntry> = Vec::with_capacity(entries.len());
    let mut seen: Vec<(u32, u32, u16)> = Vec::with_capacity(entries.len());

    for e in entries.drain(..) {
        let (w, h) = e.dimensions();
        let key = (w, h, e.bpp);
        if seen.iter().any(|k| *k == key) {
            continue;
        }
        if !preferred_sizes.iter().any(|&s| s == (w, h)) && kept.len() >= 4 {
            continue;
        }
        seen.push(key);
        kept.push(e);
    }

    if !kept.is_empty() {
        *entries = kept;
    }
}

fn apply_optimize_embedded(entries: &mut [IcoEntry], quality: u8, _config: &ImageOptConfig) -> PixieResult<()> {
    #[cfg(feature = "image")]
    for e in entries.iter_mut() {
        if !e.is_png() {
            continue;
        }
        if let Ok(img) = image::load_from_memory_with_format(&e.image_data, image::ImageFormat::Png) {
            let mut buffer: Vec<u8> = Vec::new();
            let compression = if quality <= 50 {
                image::codecs::png::CompressionType::Best
            } else {
                image::codecs::png::CompressionType::Default
            };
            let encoder = image::codecs::png::PngEncoder::new_with_quality(
                &mut buffer,
                compression,
                image::codecs::png::FilterType::Adaptive,
            );
            if img.write_with_encoder(encoder).is_ok() && buffer.len() < e.image_data.len() {
                e.image_data = buffer;
            }
        }
    }

    #[cfg(not(feature = "image"))]
    {
        let _ = (entries, quality);
    }

    Ok(())
}

fn apply_modern_recompression(entries: &mut [IcoEntry]) -> PixieResult<()> {
    #[cfg(feature = "image")]
    for e in entries.iter_mut() {
        if e.is_png() {
            continue;
        }
        if let Some(bmp_file) = wrap_dib_as_bmp(&e.image_data) {
            if let Ok(img) = image::load_from_memory_with_format(&bmp_file, image::ImageFormat::Bmp) {
                let mut buffer: Vec<u8> = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut buffer,
                    image::codecs::png::CompressionType::Best,
                    image::codecs::png::FilterType::Adaptive,
                );
                if img.write_with_encoder(encoder).is_ok() && buffer.len() < e.image_data.len() {
                    e.image_data = buffer;
                }
            }
        }
    }

    #[cfg(not(feature = "image"))]
    {
        let _ = entries;
    }

    Ok(())
}

#[cfg(feature = "image")]
fn wrap_dib_as_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    if dib.len() < 40 {
        return None;
    }
    let dib_header_size = u32::from_le_bytes([dib[0], dib[1], dib[2], dib[3]]) as usize;
    if dib_header_size < 40 || dib_header_size > dib.len() {
        return None;
    }

    let bpp = u16::from_le_bytes([dib[14], dib[15]]) as usize;
    let colors_used = u32::from_le_bytes([dib[32], dib[33], dib[34], dib[35]]) as usize;
    let palette_entries = if bpp <= 8 {
        if colors_used == 0 { 1 << bpp } else { colors_used }
    } else {
        0
    };

    let height_raw = i32::from_le_bytes([dib[8], dib[9], dib[10], dib[11]]);
    let actual_height = (height_raw.unsigned_abs() / 2) as usize;
    let dib_with_correct_height = {
        let mut adjusted = dib.to_vec();
        let h_bytes = (actual_height as i32).to_le_bytes();
        adjusted[8..12].copy_from_slice(&h_bytes);
        adjusted
    };

    let palette_bytes = palette_entries * 4;
    let pixel_offset = 14 + dib_header_size + palette_bytes;
    let total_size = pixel_offset + dib_with_correct_height.len().saturating_sub(dib_header_size + palette_bytes);

    let mut out = Vec::with_capacity(total_size);
    out.push(b'B');
    out.push(b'M');
    out.extend_from_slice(&(total_size as u32).to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&(pixel_offset as u32).to_le_bytes());
    out.extend_from_slice(&dib_with_correct_height);

    Some(out)
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
        _ => 32,
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

    Ok(u16::from_le_bytes([data[4], data[5]]))
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
        let entry_offset = 6 + i * 16;
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
        let ico_header = [0x00, 0x00, 0x01, 0x00, 0x03, 0x00];
        assert_eq!(count_ico_icons(&ico_header).unwrap(), 3);
    }

    #[test]
    fn test_png_chunk_stripping() {
        let mut png = PNG_MAGIC.to_vec();
        let ihdr_data = [0u8; 13];
        png.extend_from_slice(&(ihdr_data.len() as u32).to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&ihdr_data);
        png.extend_from_slice(&[0u8; 4]);

        let text_data = b"Author\0Pixie";
        png.extend_from_slice(&(text_data.len() as u32).to_be_bytes());
        png.extend_from_slice(b"tEXt");
        png.extend_from_slice(text_data);
        png.extend_from_slice(&[0u8; 4]);

        png.extend_from_slice(&0u32.to_be_bytes());
        png.extend_from_slice(b"IEND");
        png.extend_from_slice(&[0u8; 4]);

        let stripped = strip_ancillary_png_chunks(&png).unwrap();
        assert!(stripped.len() < png.len());
        assert!(stripped.windows(4).any(|w| w == b"IHDR"));
        assert!(stripped.windows(4).any(|w| w == b"IEND"));
        assert!(!stripped.windows(4).any(|w| w == b"tEXt"));
    }
}
