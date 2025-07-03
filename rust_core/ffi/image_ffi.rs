/// FFI bindings for image processing C hotspots
/// Provides color quantization, dithering, and convolution operations

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
pub struct QuantizedImage {
    pub palette: *mut Color32,
    pub palette_size: usize,
    pub indices: *mut u8,
    pub width: usize,
    pub height: usize,
}

// Conditionally compile C FFI declarations only when c_hotspots feature is enabled
#[cfg(feature = "c_hotspots")]
extern "C" {
    // Color quantization algorithms
    fn quantize_colors_octree(
        rgba_data: *const u8,
        width: usize,
        height: usize,
        max_colors: usize,
    ) -> *mut QuantizedImage;
    
    fn quantize_colors_median_cut(
        rgba_data: *const u8,
        width: usize,
        height: usize,
        max_colors: usize,
    ) -> *mut QuantizedImage;
    
    // Dithering algorithms
    fn apply_floyd_steinberg_dither(
        rgba_data: *mut u8,
        width: usize,
        height: usize,
        palette: *const Color32,
        palette_size: usize,
    );
    
    fn apply_ordered_dither(
        rgba_data: *mut u8,
        width: usize,
        height: usize,
        palette: *const Color32,
        palette_size: usize,
        matrix_size: i32,
    );
    
    // Convolution filters
    fn apply_gaussian_blur(
        rgba_data: *mut u8,
        width: usize,
        height: usize,
        sigma: f32,
    );
    
    fn apply_sharpen_filter(
        rgba_data: *mut u8,
        width: usize,
        height: usize,
        strength: f32,
    );
    
    fn apply_edge_detection(
        rgba_data: *const u8,
        width: usize,
        height: usize,
        output: *mut u8,
    );
    
    // Color space conversions
    fn rgb_to_yuv(rgb: *const u8, yuv: *mut u8, pixel_count: usize);
    fn yuv_to_rgb(yuv: *const u8, rgb: *mut u8, pixel_count: usize);
    fn rgb_to_lab(rgb: *const u8, lab: *mut f32, pixel_count: usize);
    fn lab_to_rgb(lab: *const f32, rgb: *mut u8, pixel_count: usize);
    
    // Memory management
    fn free_quantized_image(img: *mut QuantizedImage);
}

/// Safe wrapper for color quantization using octree algorithm
pub fn quantize_colors_octree_safe(
    rgba_data: &[u8],
    width: usize,
    height: usize,
    max_colors: usize,
) -> Option<QuantizedImageWrapper> {
    if rgba_data.len() != width * height * 4 {
        return None;
    }
    
    #[cfg(feature = "c_hotspots")]
    {
        let result = unsafe {
            quantize_colors_octree(rgba_data.as_ptr(), width, height, max_colors)
        };
        
        if result.is_null() {
            None
        } else {
            Some(QuantizedImageWrapper { inner: result })
        }
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - simple color reduction
        let palette = vec![
            Color32 { r: 0, g: 0, b: 0, a: 255 },
            Color32 { r: 255, g: 255, b: 255, a: 255 },
        ];
        let indices = vec![0u8; width * height];
        
        let quantized_img = Box::into_raw(Box::new(QuantizedImage {
            palette: Box::into_raw(palette.into_boxed_slice()) as *mut Color32,
            palette_size: 2,
            indices: Box::into_raw(indices.into_boxed_slice()) as *mut u8,
            width,
            height,
        }));
        
        Some(QuantizedImageWrapper { inner: quantized_img })
    }
}

/// Safe wrapper for color quantization using median cut algorithm
pub fn quantize_colors_median_cut_safe(
    rgba_data: &[u8],
    width: usize,
    height: usize,
    max_colors: usize,
) -> Option<QuantizedImageWrapper> {
    if rgba_data.len() != width * height * 4 {
        return None;
    }
    
    #[cfg(feature = "c_hotspots")]
    {
        let result = unsafe {
            quantize_colors_median_cut(rgba_data.as_ptr(), width, height, max_colors)
        };
        
        if result.is_null() {
            None
        } else {
            Some(QuantizedImageWrapper { inner: result })
        }
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - same as octree for now
        quantize_colors_octree_safe(rgba_data, width, height, max_colors)
    }
}

/// Safe wrapper for Floyd-Steinberg dithering
pub fn apply_floyd_steinberg_dither_safe(
    rgba_data: &mut [u8],
    width: usize,
    height: usize,
    palette: &[Color32],
) {
    if rgba_data.len() != width * height * 4 || palette.is_empty() {
        return;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        apply_floyd_steinberg_dither(
            rgba_data.as_mut_ptr(),
            width,
            height,
            palette.as_ptr(),
            palette.len(),
        );
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - no-op for now
        // In a real implementation, you'd implement Floyd-Steinberg dithering in Rust
    }
}

/// Safe wrapper for ordered dithering
pub fn apply_ordered_dither_safe(
    rgba_data: &mut [u8],
    width: usize,
    height: usize,
    palette: &[Color32],
    matrix_size: i32,
) {
    if rgba_data.len() != width * height * 4 || palette.is_empty() {
        return;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        apply_ordered_dither(
            rgba_data.as_mut_ptr(),
            width,
            height,
            palette.as_ptr(),
            palette.len(),
            matrix_size,
        );
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - no-op for now
    }
}

/// Safe wrapper for Gaussian blur
pub fn apply_gaussian_blur_safe(
    rgba_data: &mut [u8],
    width: usize,
    height: usize,
    sigma: f32,
) {
    if rgba_data.len() != width * height * 4 {
        return;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        apply_gaussian_blur(rgba_data.as_mut_ptr(), width, height, sigma);
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - no-op for now
    }
}

/// Safe wrapper for sharpen filter
pub fn apply_sharpen_filter_safe(
    rgba_data: &mut [u8],
    width: usize,
    height: usize,
    strength: f32,
) {
    if rgba_data.len() != width * height * 4 {
        return;
    }
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        apply_sharpen_filter(rgba_data.as_mut_ptr(), width, height, strength);
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - no-op for now
    }
}

/// Safe wrapper for edge detection
pub fn apply_edge_detection_safe(
    rgba_data: &[u8],
    width: usize,
    height: usize,
) -> Option<Vec<u8>> {
    if rgba_data.len() != width * height * 4 {
        return None;
    }
    
    #[cfg(feature = "c_hotspots")]
    {
        let mut output = vec![0u8; width * height];
        unsafe {
            apply_edge_detection(rgba_data.as_ptr(), width, height, output.as_mut_ptr());
        }
        Some(output)
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - return empty edge map
        Some(vec![0u8; width * height])
    }
}

/// Safe wrapper for RGB to YUV conversion
pub fn rgb_to_yuv_safe(rgb_data: &[u8]) -> Vec<u8> {
    if rgb_data.len() % 3 != 0 {
        return Vec::new();
    }
    
    let pixel_count = rgb_data.len() / 3;
    let mut yuv_data = vec![0u8; rgb_data.len()];
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        rgb_to_yuv(rgb_data.as_ptr(), yuv_data.as_mut_ptr(), pixel_count);
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - copy RGB as-is for now
        yuv_data.copy_from_slice(rgb_data);
    }
    
    yuv_data
}

/// Safe wrapper for YUV to RGB conversion
pub fn yuv_to_rgb_safe(yuv_data: &[u8]) -> Vec<u8> {
    if yuv_data.len() % 3 != 0 {
        return Vec::new();
    }
    
    let pixel_count = yuv_data.len() / 3;
    let mut rgb_data = vec![0u8; yuv_data.len()];
    
    #[cfg(feature = "c_hotspots")]
    unsafe {
        yuv_to_rgb(yuv_data.as_ptr(), rgb_data.as_mut_ptr(), pixel_count);
    }
    
    #[cfg(not(feature = "c_hotspots"))]
    {
        // Rust stub implementation - copy YUV as-is for now
        rgb_data.copy_from_slice(yuv_data);
    }
    
    rgb_data
}

/// RAII wrapper for QuantizedImage
pub struct QuantizedImageWrapper {
    inner: *mut QuantizedImage,
}

impl QuantizedImageWrapper {
    pub fn get_palette(&self) -> &[Color32] {
        unsafe {
            let img = &*self.inner;
            if img.palette.is_null() || img.palette_size == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(img.palette, img.palette_size)
            }
        }
    }
    
    pub fn get_indices(&self) -> &[u8] {
        unsafe {
            let img = &*self.inner;
            if img.indices.is_null() || img.width == 0 || img.height == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(img.indices, img.width * img.height)
            }
        }
    }
    
    pub fn dimensions(&self) -> (usize, usize) {
        unsafe {
            let img = &*self.inner;
            (img.width, img.height)
        }
    }
    
    pub fn palette_size(&self) -> usize {
        unsafe {
            let img = &*self.inner;
            img.palette_size
        }
    }
}

impl Drop for QuantizedImageWrapper {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            #[cfg(feature = "c_hotspots")]
            unsafe {
                free_quantized_image(self.inner);
            }
            
            #[cfg(not(feature = "c_hotspots"))]
            unsafe {
                // Free Rust-allocated memory
                let img = Box::from_raw(self.inner);
                if !img.palette.is_null() {
                    let _ = Box::from_raw(std::slice::from_raw_parts_mut(img.palette, img.palette_size));
                }
                if !img.indices.is_null() {
                    let _ = Box::from_raw(std::slice::from_raw_parts_mut(img.indices, img.width * img.height));
                }
            }
        }
    }
}

unsafe impl Send for QuantizedImageWrapper {}
unsafe impl Sync for QuantizedImageWrapper {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_quantization_api() {
        let rgba_data = vec![255u8; 4 * 4 * 4]; // 4x4 white image
        
        let result = quantize_colors_octree_safe(&rgba_data, 4, 4, 16);
        assert!(result.is_some());
        
        if let Some(quantized) = result {
            assert_eq!(quantized.dimensions(), (4, 4));
            assert!(quantized.palette_size() <= 16);
            assert_eq!(quantized.get_indices().len(), 16);
        }
    }
    
    #[test]
    fn test_dithering_api() {
        let mut rgba_data = vec![128u8; 4 * 4 * 4]; // 4x4 gray image
        let palette = vec![
            Color32 { r: 0, g: 0, b: 0, a: 255 },
            Color32 { r: 255, g: 255, b: 255, a: 255 },
        ];
        
        // Should not panic
        apply_floyd_steinberg_dither_safe(&mut rgba_data, 4, 4, &palette);
        apply_ordered_dither_safe(&mut rgba_data, 4, 4, &palette, 4);
    }
    
    #[test]
    fn test_color_conversion_api() {
        let rgb_data = vec![255, 128, 64, 255, 128, 64]; // 2 pixels
        
        let yuv_data = rgb_to_yuv_safe(&rgb_data);
        assert_eq!(yuv_data.len(), rgb_data.len());
        
        let converted_back = yuv_to_rgb_safe(&yuv_data);
        assert_eq!(converted_back.len(), rgb_data.len());
    }
}
