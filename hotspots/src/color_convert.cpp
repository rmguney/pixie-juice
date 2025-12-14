#include "image_kernel.h"

static inline uint8_t clamp_u8_i32(int32_t v) {
    if (v < 0) return 0;
    if (v > 255) return 255;
    return (uint8_t)v;
}

extern "C" {

WASM_EXPORT void rgb_to_yuv(const uint8_t* rgb, uint8_t* yuv, size_t pixel_count) {
    if (!rgb || !yuv || pixel_count == 0) {
        return;
    }

    for (size_t i = 0; i < pixel_count; i++) {
        const uint32_t r = rgb[i * 3 + 0];
        const uint32_t g = rgb[i * 3 + 1];
        const uint32_t b = rgb[i * 3 + 2];

        const int32_t y = (int32_t)((77u * r + 150u * g + 29u * b + 128u) >> 8);
        const int32_t u = (int32_t)((((int32_t)(-43) * (int32_t)r + (int32_t)(-85) * (int32_t)g + (int32_t)(128) * (int32_t)b + 128) >> 8) + 128);
        const int32_t v = (int32_t)((((int32_t)(128) * (int32_t)r + (int32_t)(-107) * (int32_t)g + (int32_t)(-21) * (int32_t)b + 128) >> 8) + 128);

        yuv[i * 3 + 0] = clamp_u8_i32(y);
        yuv[i * 3 + 1] = clamp_u8_i32(u);
        yuv[i * 3 + 2] = clamp_u8_i32(v);
    }
}

WASM_EXPORT void yuv_to_rgb(const uint8_t* yuv, uint8_t* rgb, size_t pixel_count) {
    if (!yuv || !rgb || pixel_count == 0) {
        return;
    }

    for (size_t i = 0; i < pixel_count; i++) {
        const int32_t y = (int32_t)yuv[i * 3 + 0];
        const int32_t u = (int32_t)yuv[i * 3 + 1] - 128;
        const int32_t v = (int32_t)yuv[i * 3 + 2] - 128;

        const int32_t r = y + ((359 * v + 128) >> 8);
        const int32_t g = y - ((88 * u + 183 * v + 128) >> 8);
        const int32_t b = y + ((454 * u + 128) >> 8);

        rgb[i * 3 + 0] = clamp_u8_i32(r);
        rgb[i * 3 + 1] = clamp_u8_i32(g);
        rgb[i * 3 + 2] = clamp_u8_i32(b);
    }
}

WASM_EXPORT void rgba_yuv_roundtrip_inplace(uint8_t* rgba, size_t pixel_count) {
    if (!rgba || pixel_count == 0) {
        return;
    }

    for (size_t i = 0; i < pixel_count; i++) {
        const uint32_t r0 = rgba[i * 4 + 0];
        const uint32_t g0 = rgba[i * 4 + 1];
        const uint32_t b0 = rgba[i * 4 + 2];

        const int32_t y = (int32_t)((77u * r0 + 150u * g0 + 29u * b0 + 128u) >> 8);
        const int32_t u = (int32_t)((((int32_t)(-43) * (int32_t)r0 + (int32_t)(-85) * (int32_t)g0 + (int32_t)(128) * (int32_t)b0 + 128) >> 8) + 128);
        const int32_t v = (int32_t)((((int32_t)(128) * (int32_t)r0 + (int32_t)(-107) * (int32_t)g0 + (int32_t)(-21) * (int32_t)b0 + 128) >> 8) + 128);

        const int32_t uu = u - 128;
        const int32_t vv = v - 128;

        const int32_t r = y + ((359 * vv + 128) >> 8);
        const int32_t g = y - ((88 * uu + 183 * vv + 128) >> 8);
        const int32_t b = y + ((454 * uu + 128) >> 8);

        rgba[i * 4 + 0] = clamp_u8_i32(r);
        rgba[i * 4 + 1] = clamp_u8_i32(g);
        rgba[i * 4 + 2] = clamp_u8_i32(b);
    }
}

}
