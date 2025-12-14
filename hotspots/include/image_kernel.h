#ifndef IMAGE_KERNEL_H
#define IMAGE_KERNEL_H

#include "memory.h"
#include "util.h"

#ifdef __cplusplus
extern "C" {
#endif

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

WASM_EXPORT QuantizedImage* quantize_colors_octree(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    size_t max_colors
);

WASM_EXPORT QuantizedImage* quantize_colors_median_cut(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    size_t max_colors
);

WASM_EXPORT int apply_floyd_steinberg_dither(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const Color32* palette,
    size_t palette_size
);

WASM_EXPORT void apply_ordered_dither(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const Color32* palette,
    size_t palette_size,
    int matrix_size
);

WASM_EXPORT void apply_gaussian_blur(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    size_t channels,
    float sigma
);

WASM_EXPORT void apply_sharpen_filter(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    float strength
);

WASM_EXPORT void apply_edge_detection(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t* output
);

WASM_EXPORT void rgb_to_yuv(const uint8_t* rgb, uint8_t* yuv, size_t pixel_count);
WASM_EXPORT void yuv_to_rgb(const uint8_t* yuv, uint8_t* rgb, size_t pixel_count);
WASM_EXPORT void rgb_to_lab(const uint8_t* rgb, float* lab, size_t pixel_count);
WASM_EXPORT void lab_to_rgb(const float* lab, uint8_t* rgb, size_t pixel_count);

typedef struct {
    uint8_t* data;
    size_t size;
    uint32_t width;
    uint32_t height;
    uint8_t bits_per_sample;
    uint8_t compression;
} TIFFProcessResult;

WASM_EXPORT TIFFProcessResult* compress_tiff_lzw_simd(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t quality
);

WASM_EXPORT TIFFProcessResult* strip_tiff_metadata_simd_c_hotspot(
    const uint8_t* tiff_data,
    size_t data_size,
    bool preserve_icc
);

WASM_EXPORT void apply_tiff_predictor_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t predictor_type
);

WASM_EXPORT void optimize_tiff_colorspace_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t target_bits_per_channel
);

WASM_EXPORT void free_quantized_image(QuantizedImage* img);
WASM_EXPORT void free_tiff_result(TIFFProcessResult* result);

WASM_EXPORT void batch_process_pixels_simd(
    uint8_t* rgba_data,
    size_t pixel_count,
    uint8_t operation_type
);

WASM_EXPORT void parallel_color_conversion_simd(
    const uint8_t* src_data,
    uint8_t* dst_data,
    size_t pixel_count,
    uint8_t src_format,
    uint8_t dst_format
);

WASM_EXPORT void vectorized_filter_apply_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const float* kernel,
    size_t kernel_size
);

WASM_EXPORT void fast_downscale_simd(
    const uint8_t* src_data,
    uint8_t* dst_data,
    size_t src_width,
    size_t src_height,
    size_t dst_width,
    size_t dst_height
);

WASM_EXPORT void multi_threaded_compression_simd(
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

#endif
