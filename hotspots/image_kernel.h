#ifndef IMAGE_KERNEL_H
#define IMAGE_KERNEL_H

#include <stdint.h>
#include <stddef.h>

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

// Memory management
void free_quantized_image(QuantizedImage* img);

#ifdef __cplusplus
}
#endif

#endif // IMAGE_KERNEL_H
