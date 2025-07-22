/// FFI bindings for image processing C hotspots
/// Full working implementations with Rust fallbacks

use crate::types::{OptResult, OptError};
use std::collections::HashMap;

/// Color representation for image processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color32 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }
}

/// Wrapper for quantized image data
#[derive(Debug)]
pub struct QuantizedImageWrapper {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub palette: Vec<Color32>,
}

impl QuantizedImageWrapper {
    pub fn new(data: Vec<u8>, width: u32, height: u32, palette: Vec<Color32>) -> Self {
        Self { data, width, height, palette }
    }
}

/// Octree node for color quantization with alpha support
struct OctreeNode {
    children: [Option<Box<OctreeNode>>; 16], // 16 children for RGBA (4 bits each channel)
    color_sum: [u64; 4], // RGBA sums
    pixel_count: u64,
    is_leaf: bool,
}

impl OctreeNode {
    fn new() -> Self {
        Self {
            children: Default::default(),
            color_sum: [0; 4], // RGBA
            pixel_count: 0,
            is_leaf: false,
        }
    }
    
    fn add_color(&mut self, r: u8, g: u8, b: u8, a: u8, level: u8) {
        self.color_sum[0] += r as u64;
        self.color_sum[1] += g as u64;
        self.color_sum[2] += b as u64;
        self.color_sum[3] += a as u64;
        self.pixel_count += 1;
        
        if level >= 6 { // Reduced depth for RGBA to manage memory
            self.is_leaf = true;
            return;
        }
        
        // Use 2 bits per channel for indexing (4 channels = 8 bits = 256 possible indices, but we use 16)
        let shift = 7 - (level * 2);
        let index = ((r >> shift) & 3) << 6 |
                   ((g >> shift) & 3) << 4 |
                   ((b >> shift) & 3) << 2 |
                   ((a >> shift) & 3);
        
        let index = (index % 16) as usize; // Ensure we stay within bounds
        
        if self.children[index].is_none() {
            self.children[index] = Some(Box::new(OctreeNode::new()));
        }
        
        self.children[index].as_mut().unwrap().add_color(r, g, b, a, level + 1);
    }
    
    fn get_palette(&self, palette: &mut Vec<Color32>) {
        if self.is_leaf || self.pixel_count == 0 {
            if self.pixel_count > 0 {
                let r = (self.color_sum[0] / self.pixel_count) as u8;
                let g = (self.color_sum[1] / self.pixel_count) as u8;
                let b = (self.color_sum[2] / self.pixel_count) as u8;
                let a = (self.color_sum[3] / self.pixel_count) as u8;
                palette.push(Color32::new(r, g, b, a));
            }
            return;
        }
        
        for child in &self.children {
            if let Some(child) = child {
                child.get_palette(palette);
            }
        }
    }
}

/// Full working implementation for color quantization using octree
pub fn quantize_colors_octree_safe(
    image_data: &[u8],
    width: u32,
    height: u32,
    max_colors: u32,
) -> OptResult<QuantizedImageWrapper> {
    if image_data.len() < (width * height * 4) as usize {
        return Err(OptError::InvalidInput("Image data too small".to_string()));
    }
    
    // Build octree - preserve alpha channel information
    let mut root = OctreeNode::new();
    for chunk in image_data.chunks_exact(4) {
        root.add_color(chunk[0], chunk[1], chunk[2], chunk[3], 0);
    }
    
    // Extract palette
    let mut palette = Vec::new();
    root.get_palette(&mut palette);
    
    // Limit palette size
    if palette.len() > max_colors as usize {
        palette.truncate(max_colors as usize);
    }
    
    // Create color map for fast lookup with RGBA colors
    let mut color_map = HashMap::new();
    for (i, &color) in palette.iter().enumerate() {
        color_map.insert(color, i as u8);
    }
    
    // Quantize image - preserve alpha channel
    let mut quantized_data = Vec::with_capacity((width * height) as usize);
    for chunk in image_data.chunks_exact(4) {
        let color = Color32::new(chunk[0], chunk[1], chunk[2], chunk[3]); // Include alpha
        let index = color_map.get(&color).copied().unwrap_or_else(|| {
            // Find nearest color if exact match not found, but preserve alpha
            find_nearest_color_with_alpha(&color, &palette)
        });
        quantized_data.push(index);
    }
    
    Ok(QuantizedImageWrapper::new(quantized_data, width, height, palette))
}

fn find_nearest_color_with_alpha(target: &Color32, palette: &[Color32]) -> u8 {
    let mut best_index = 0;
    let mut best_distance = u32::MAX;
    
    for (i, &color) in palette.iter().enumerate() {
        let dr = (target.r as i32 - color.r as i32).abs() as u32;
        let dg = (target.g as i32 - color.g as i32).abs() as u32;
        let db = (target.b as i32 - color.b as i32).abs() as u32;
        let da = (target.a as i32 - color.a as i32).abs() as u32;
        // Weight alpha channel more heavily to preserve transparency
        let distance = dr * dr + dg * dg + db * db + (da * da * 2);
        
        if distance < best_distance {
            best_distance = distance;
            best_index = i;
        }
    }
    
    best_index as u8
}

/// Full working implementation for color quantization using median cut
pub fn quantize_colors_median_cut_safe(
    image_data: &[u8],
    width: u32,
    height: u32,
    max_colors: u32,
) -> OptResult<QuantizedImageWrapper> {
    if image_data.len() < (width * height * 4) as usize {
        return Err(OptError::InvalidInput("Image data too small".to_string()));
    }
    
    // Collect unique colors with alpha
    let mut colors: Vec<Color32> = image_data
        .chunks_exact(4)
        .map(|chunk| Color32::new(chunk[0], chunk[1], chunk[2], chunk[3])) // Include alpha
        .collect();
    
    colors.sort_unstable_by_key(|c| (c.r as u32) << 24 | (c.g as u32) << 16 | (c.b as u32) << 8 | c.a as u32);
    colors.dedup();
    
    // Apply median cut algorithm
    let palette = median_cut(&mut colors, max_colors as usize);
    
    // Create color map for fast lookup
    let mut color_map = HashMap::new();
    for (i, &color) in palette.iter().enumerate() {
        color_map.insert(color, i as u8);
    }
    
    // Quantize image preserving alpha
    let mut quantized_data = Vec::with_capacity((width * height) as usize);
    for chunk in image_data.chunks_exact(4) {
        let color = Color32::new(chunk[0], chunk[1], chunk[2], chunk[3]); // Include alpha
        let index = color_map.get(&color).copied().unwrap_or_else(|| {
            find_nearest_color_with_alpha(&color, &palette)
        });
        quantized_data.push(index);
    }
    
    Ok(QuantizedImageWrapper::new(quantized_data, width, height, palette))
}

fn median_cut(colors: &mut [Color32], max_colors: usize) -> Vec<Color32> {
    if colors.is_empty() {
        return Vec::new();
    }
    
    if colors.len() <= max_colors {
        return colors.to_vec();
    }
    
    // Simple median cut implementation
    let mut buckets = vec![colors.to_vec()];
    
    while buckets.len() < max_colors && buckets.iter().any(|b| b.len() > 1) {
        let mut new_buckets = Vec::new();
        
        for bucket in buckets {
            if bucket.len() <= 1 {
                new_buckets.push(bucket);
                continue;
            }
            
            // Find dimension with largest range (including alpha)
            let (min_r, max_r) = bucket.iter().map(|c| c.r).fold((255, 0), |(min, max), r| (min.min(r), max.max(r)));
            let (min_g, max_g) = bucket.iter().map(|c| c.g).fold((255, 0), |(min, max), g| (min.min(g), max.max(g)));
            let (min_b, max_b) = bucket.iter().map(|c| c.b).fold((255, 0), |(min, max), b| (min.min(b), max.max(b)));
            let (min_a, max_a) = bucket.iter().map(|c| c.a).fold((255, 0), |(min, max), a| (min.min(a), max.max(a)));
            
            let r_range = max_r - min_r;
            let g_range = max_g - min_g;
            let b_range = max_b - min_b;
            let a_range = max_a - min_a;
            
            let mut sorted_bucket = bucket;
            if a_range >= r_range && a_range >= g_range && a_range >= b_range {
                // Alpha has largest range - sort by alpha to preserve transparency
                sorted_bucket.sort_unstable_by_key(|c| c.a);
            } else if r_range >= g_range && r_range >= b_range {
                sorted_bucket.sort_unstable_by_key(|c| c.r);
            } else if g_range >= b_range {
                sorted_bucket.sort_unstable_by_key(|c| c.g);
            } else {
                sorted_bucket.sort_unstable_by_key(|c| c.b);
            }
            
            let mid = sorted_bucket.len() / 2;
            new_buckets.push(sorted_bucket[..mid].to_vec());
            new_buckets.push(sorted_bucket[mid..].to_vec());
        }
        
        buckets = new_buckets;
    }
    
    // Calculate average color for each bucket (including alpha)
    buckets.into_iter().map(|bucket| {
        let r_sum: u32 = bucket.iter().map(|c| c.r as u32).sum();
        let g_sum: u32 = bucket.iter().map(|c| c.g as u32).sum();
        let b_sum: u32 = bucket.iter().map(|c| c.b as u32).sum();
        let a_sum: u32 = bucket.iter().map(|c| c.a as u32).sum();
        let count = bucket.len() as u32;
        
        Color32::new(
            (r_sum / count) as u8,
            (g_sum / count) as u8,
            (b_sum / count) as u8,
            (a_sum / count) as u8,
        )
    }).collect()
}

/// Full working implementation for Floyd-Steinberg dithering
pub fn apply_floyd_steinberg_dither_safe(
    image_data: &[u8],
    width: u32,
    height: u32,
) -> OptResult<Vec<u8>> {
    if image_data.len() < (width * height * 4) as usize {
        return Err(OptError::InvalidInput("Image data too small".to_string()));
    }
    
    let mut result = image_data.to_vec();
    let w = width as usize;
    let h = height as usize;
    
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            
            // Quantize to black or white
            let old_r = result[idx] as f32;
            let old_g = result[idx + 1] as f32;
            let old_b = result[idx + 2] as f32;
            
            let gray = (old_r * 0.299 + old_g * 0.587 + old_b * 0.114) / 255.0;
            let new_gray = if gray > 0.5 { 255.0 } else { 0.0 };
            
            result[idx] = new_gray as u8;
            result[idx + 1] = new_gray as u8;
            result[idx + 2] = new_gray as u8;
            
            let error = gray * 255.0 - new_gray;
            
            // Distribute error
            if x + 1 < w {
                let right_idx = (y * w + x + 1) * 4;
                apply_error(&mut result, right_idx, error * 7.0 / 16.0);
            }
            
            if y + 1 < h {
                if x > 0 {
                    let bottom_left_idx = ((y + 1) * w + x - 1) * 4;
                    apply_error(&mut result, bottom_left_idx, error * 3.0 / 16.0);
                }
                
                let bottom_idx = ((y + 1) * w + x) * 4;
                apply_error(&mut result, bottom_idx, error * 5.0 / 16.0);
                
                if x + 1 < w {
                    let bottom_right_idx = ((y + 1) * w + x + 1) * 4;
                    apply_error(&mut result, bottom_right_idx, error * 1.0 / 16.0);
                }
            }
        }
    }
    
    Ok(result)
}

fn apply_error(data: &mut [u8], idx: usize, error: f32) {
    for i in 0..3 {
        let new_val = (data[idx + i] as f32 + error).clamp(0.0, 255.0) as u8;
        data[idx + i] = new_val;
    }
}

/// Full working implementation for ordered dithering
pub fn apply_ordered_dither_safe(
    image_data: &[u8],
    width: u32,
    height: u32,
) -> OptResult<Vec<u8>> {
    if image_data.len() < (width * height * 4) as usize {
        return Err(OptError::InvalidInput("Image data too small".to_string()));
    }
    
    // Bayer matrix 4x4
    const BAYER_MATRIX: [[u8; 4]; 4] = [
        [0, 8, 2, 10],
        [12, 4, 14, 6],
        [3, 11, 1, 9],
        [15, 7, 13, 5],
    ];
    
    let mut result = image_data.to_vec();
    let w = width as usize;
    let h = height as usize;
    
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            
            let threshold = (BAYER_MATRIX[y % 4][x % 4] as f32 / 16.0) * 255.0;
            
            for i in 0..3 {
                let value = result[idx + i] as f32;
                result[idx + i] = if value > threshold { 255 } else { 0 };
            }
        }
    }
    
    Ok(result)
}

/// Full working implementation for Gaussian blur
pub fn apply_gaussian_blur_safe(
    image_data: &[u8],
    width: u32,
    height: u32,
    sigma: f32,
) -> OptResult<Vec<u8>> {
    if image_data.len() < (width * height * 4) as usize {
        return Err(OptError::InvalidInput("Image data too small".to_string()));
    }
    
    let radius = (sigma * 3.0).ceil() as i32;
    let mut kernel = Vec::new();
    let mut kernel_sum = 0.0;
    
    // Generate Gaussian kernel
    for i in -radius..=radius {
        let weight = (-((i * i) as f32) / (2.0 * sigma * sigma)).exp();
        kernel.push(weight);
        kernel_sum += weight;
    }
    
    // Normalize kernel
    for weight in &mut kernel {
        *weight /= kernel_sum;
    }
    
    let w = width as usize;
    let h = height as usize;
    let mut result = vec![0u8; image_data.len()];
    
    // Horizontal pass
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            
            for c in 0..4 {
                let mut sum = 0.0;
                
                for (ki, &weight) in kernel.iter().enumerate() {
                    let kx = x as i32 + ki as i32 - radius;
                    let kx = kx.clamp(0, w as i32 - 1) as usize;
                    let kidx = (y * w + kx) * 4;
                    sum += image_data[kidx + c] as f32 * weight;
                }
                
                result[idx + c] = sum.clamp(0.0, 255.0) as u8;
            }
        }
    }
    
    // Vertical pass
    let mut final_result = vec![0u8; image_data.len()];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            
            for c in 0..4 {
                let mut sum = 0.0;
                
                for (ki, &weight) in kernel.iter().enumerate() {
                    let ky = y as i32 + ki as i32 - radius;
                    let ky = ky.clamp(0, h as i32 - 1) as usize;
                    let kidx = (ky * w + x) * 4;
                    sum += result[kidx + c] as f32 * weight;
                }
                
                final_result[idx + c] = sum.clamp(0.0, 255.0) as u8;
            }
        }
    }
    
    Ok(final_result)
}

/// Full working implementation for sharpen filter
pub fn apply_sharpen_filter_safe(
    image_data: &[u8],
    width: u32,
    height: u32,
) -> OptResult<Vec<u8>> {
    if image_data.len() < (width * height * 4) as usize {
        return Err(OptError::InvalidInput("Image data too small".to_string()));
    }
    
    // Sharpen kernel
    const KERNEL: [[f32; 3]; 3] = [
        [0.0, -1.0, 0.0],
        [-1.0, 5.0, -1.0],
        [0.0, -1.0, 0.0],
    ];
    
    let w = width as usize;
    let h = height as usize;
    let mut result = image_data.to_vec();
    
    for y in 1..h-1 {
        for x in 1..w-1 {
            let idx = (y * w + x) * 4;
            
            for c in 0..3 { // Skip alpha channel
                let mut sum = 0.0;
                
                for ky in 0..3 {
                    for kx in 0..3 {
                        let py = y + ky - 1;
                        let px = x + kx - 1;
                        let pidx = (py * w + px) * 4;
                        sum += image_data[pidx + c] as f32 * KERNEL[ky][kx];
                    }
                }
                
                result[idx + c] = sum.clamp(0.0, 255.0) as u8;
            }
        }
    }
    
    Ok(result)
}

/// Full working implementation for edge detection
pub fn apply_edge_detection_safe(
    image_data: &[u8],
    width: u32,
    height: u32,
) -> OptResult<Vec<u8>> {
    if image_data.len() < (width * height * 4) as usize {
        return Err(OptError::InvalidInput("Image data too small".to_string()));
    }
    
    // Sobel kernels
    const SOBEL_X: [[f32; 3]; 3] = [
        [-1.0, 0.0, 1.0],
        [-2.0, 0.0, 2.0],
        [-1.0, 0.0, 1.0],
    ];
    
    const SOBEL_Y: [[f32; 3]; 3] = [
        [-1.0, -2.0, -1.0],
        [0.0, 0.0, 0.0],
        [1.0, 2.0, 1.0],
    ];
    
    let w = width as usize;
    let h = height as usize;
    let mut result = vec![0u8; image_data.len()];
    
    for y in 1..h-1 {
        for x in 1..w-1 {
            let idx = (y * w + x) * 4;
            
            let mut gx = 0.0;
            let mut gy = 0.0;
            
            for ky in 0..3 {
                for kx in 0..3 {
                    let py = y + ky - 1;
                    let px = x + kx - 1;
                    let pidx = (py * w + px) * 4;
                    
                    // Convert to grayscale
                    let gray = (image_data[pidx] as f32 * 0.299 + 
                               image_data[pidx + 1] as f32 * 0.587 + 
                               image_data[pidx + 2] as f32 * 0.114) / 255.0;
                    
                    gx += gray * SOBEL_X[ky][kx];
                    gy += gray * SOBEL_Y[ky][kx];
                }
            }
            
            let magnitude = ((gx * gx + gy * gy).sqrt() * 255.0).clamp(0.0, 255.0) as u8;
            
            result[idx] = magnitude;
            result[idx + 1] = magnitude;
            result[idx + 2] = magnitude;
            result[idx + 3] = image_data[idx + 3]; // Preserve alpha
        }
    }
    
    Ok(result)
}

/// Full working implementation for RGB to YUV conversion
pub fn rgb_to_yuv_safe(
    rgb_data: &[u8],
    width: u32,
    height: u32,
) -> OptResult<Vec<u8>> {
    if rgb_data.len() < (width * height * 3) as usize {
        return Err(OptError::InvalidInput("RGB data too small".to_string()));
    }
    
    let mut yuv_data = Vec::with_capacity(rgb_data.len());
    
    for chunk in rgb_data.chunks_exact(3) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        
        let y = (0.299 * r + 0.587 * g + 0.114 * b).clamp(0.0, 255.0) as u8;
        let u = ((-0.14713 * r - 0.28886 * g + 0.436 * b) + 128.0).clamp(0.0, 255.0) as u8;
        let v = ((0.615 * r - 0.51499 * g - 0.10001 * b) + 128.0).clamp(0.0, 255.0) as u8;
        
        yuv_data.push(y);
        yuv_data.push(u);
        yuv_data.push(v);
    }
    
    Ok(yuv_data)
}

/// Full working implementation for YUV to RGB conversion
pub fn yuv_to_rgb_safe(
    yuv_data: &[u8],
    width: u32,
    height: u32,
) -> OptResult<Vec<u8>> {
    if yuv_data.len() < (width * height * 3) as usize {
        return Err(OptError::InvalidInput("YUV data too small".to_string()));
    }
    
    let mut rgb_data = Vec::with_capacity(yuv_data.len());
    
    for chunk in yuv_data.chunks_exact(3) {
        let y = chunk[0] as f32;
        let u = chunk[1] as f32 - 128.0;
        let v = chunk[2] as f32 - 128.0;
        
        let r = (y + 1.13983 * v).clamp(0.0, 255.0) as u8;
        let g = (y - 0.39465 * u - 0.58060 * v).clamp(0.0, 255.0) as u8;
        let b = (y + 2.03211 * u).clamp(0.0, 255.0) as u8;
        
        rgb_data.push(r);
        rgb_data.push(g);
        rgb_data.push(b);
    }
    
    Ok(rgb_data)
}

// C FFI declarations for when C hotspots are enabled
#[cfg(c_hotspots_available)]
extern "C" {
    pub fn quantize_colors_octree(
        image_data: *const u8, width: u32, height: u32, max_colors: u32
    ) -> *mut std::ffi::c_void;
    
    pub fn apply_gaussian_blur(
        image_data: *const u8, width: u32, height: u32, sigma: f32
    ) -> *mut std::ffi::c_void;
    
    pub fn apply_floyd_steinberg_dither(
        image_data: *const u8, width: u32, height: u32
    ) -> *mut std::ffi::c_void;
}