#ifndef IMAGE_KERNEL_H
#define IMAGE_KERNEL_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Color quantization algorithms
typedef struct {
    uint8_t r, g, b, a;
} Color32;

typedef struct {
    Color32* palette;
    size_t palette_size;
    uint8_t* indices;
    size_t width;
    size_t height;
} QuantizedImage;

// Color quantization using octree algorithm
QuantizedImage* quantize_colors_octree(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    size_t max_colors
);

// Color quantization using median cut algorithm
QuantizedImage* quantize_colors_median_cut(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    size_t max_colors
);

// Dithering algorithms
void apply_floyd_steinberg_dither(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const Color32* palette,
    size_t palette_size
);

void apply_ordered_dither(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const Color32* palette,
    size_t palette_size,
    int matrix_size
);

// Convolution filters
void apply_gaussian_blur(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    float sigma
);

void apply_sharpen_filter(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    float strength
);

void apply_edge_detection(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t* output
);

// Color space conversions
void rgb_to_yuv(const uint8_t* rgb, uint8_t* yuv, size_t pixel_count);
void yuv_to_rgb(const uint8_t* yuv, uint8_t* rgb, size_t pixel_count);
void rgb_to_lab(const uint8_t* rgb, float* lab, size_t pixel_count);
void lab_to_rgb(const float* lab, uint8_t* rgb, size_t pixel_count);

// TIFF-specific optimizations with SIMD acceleration
typedef struct {
    uint8_t* data;
    size_t size;
    uint32_t width;
    uint32_t height;
    uint8_t bits_per_sample;
    uint8_t compression;
} TIFFProcessResult;

// TIFF LZW compression with SIMD string matching
TIFFProcessResult* compress_tiff_lzw_simd(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t quality
);

// TIFF metadata stripping with SIMD tag processing
TIFFProcessResult* strip_tiff_metadata_simd(
    const uint8_t* tiff_data,
    size_t data_size,
    bool preserve_icc
);

// TIFF predictor preprocessing for better compression
void apply_tiff_predictor_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t predictor_type
);

// TIFF color space optimization
void optimize_tiff_colorspace_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t target_bits_per_channel
);

// Memory management
void free_quantized_image(QuantizedImage* img);
void free_tiff_result(TIFFProcessResult* result);

// Advanced SIMD acceleration functions for performance optimization
void batch_process_pixels_simd(
    uint8_t* rgba_data,
    size_t pixel_count,
    uint8_t operation_type
);

void parallel_color_conversion_simd(
    const uint8_t* src_data,
    uint8_t* dst_data,
    size_t pixel_count,
    uint8_t src_format,
    uint8_t dst_format
);

void vectorized_filter_apply_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const float* kernel,
    size_t kernel_size
);

void fast_downscale_simd(
    const uint8_t* src_data,
    uint8_t* dst_data,
    size_t src_width,
    size_t src_height,
    size_t dst_width,
    size_t dst_height
);

void multi_threaded_compression_simd(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t* compressed_data,
    size_t* compressed_size,
    uint8_t quality
);

#ifdef __cplusplus
}
#endif

#endif // IMAGE_KERNEL_H
