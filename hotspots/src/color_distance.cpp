#include "color_lut.hpp"
#include "util.h"

#ifdef __wasm_simd128__
#include <wasm_simd128.h>
#define SIMD_AVAILABLE 1
#else
#define SIMD_AVAILABLE 0
#endif

extern "C" {

WASM_EXPORT float color_distance_perceptual(
    unsigned char r1, unsigned char g1, unsigned char b1,
    unsigned char r2, unsigned char g2, unsigned char b2
) {
    const float* lut = get_srgb_to_linear_lut();
    float lr1 = lut[r1];
    float lg1 = lut[g1];
    float lb1 = lut[b1];
    float lr2 = lut[r2];
    float lg2 = lut[g2];
    float lb2 = lut[b2];
    
    float dr = lr1 - lr2;
    float dg = lg1 - lg2;
    float db = lb1 - lb2;
    
    return dr*dr*0.299f + dg*dg*0.587f + db*db*0.114f;
}

WASM_EXPORT void rgb_to_linear_batch(
    const unsigned char* rgb,
    float* linear,
    unsigned int count
) {
    const float* lut = get_srgb_to_linear_lut();
    for(unsigned int i = 0; i < count * 3; i++) {
        linear[i] = lut[rgb[i]];
    }
}

WASM_EXPORT void linear_to_rgb_batch(
    const float* linear,
    unsigned char* rgb,
    unsigned int count
) {
    const unsigned char* lut = get_linear_to_srgb_lut();
    for(unsigned int i = 0; i < count * 3; i++) {
        float val = linear[i];
        if (val < 0.0f) val = 0.0f;
        if (val > 1.0f) val = 1.0f;
        unsigned int index = (unsigned int)(val * 191.0f + 0.5f);
        if (index > 191) index = 191;
        rgb[i] = lut[index];
    }
}

#if SIMD_AVAILABLE
WASM_EXPORT void rgb_to_linear_batch_simd(
    const unsigned char* rgb,
    float* linear,
    unsigned int count
) {
    const float* lut = get_srgb_to_linear_lut();
    unsigned int i = 0;
    unsigned int simd_count = (count * 3) & ~3;
    
    for(; i < simd_count; i += 4) {
        float vals[4] = {
            lut[rgb[i]],
            lut[rgb[i+1]],
            lut[rgb[i+2]],
            lut[rgb[i+3]]
        };
        v128_t result = wasm_v128_load(vals);
        wasm_v128_store(&linear[i], result);
    }
    
    for(; i < count * 3; i++) {
        linear[i] = lut[rgb[i]];
    }
}
#endif

WASM_EXPORT float color_distance_batch_min(
    const unsigned char* palette,
    unsigned int palette_size,
    unsigned char r, unsigned char g, unsigned char b
) {
    float min_dist = 1e30f;
    
    for(unsigned int i = 0; i < palette_size; i++) {
        float dist = color_distance_perceptual(
            r, g, b,
            palette[i*3], palette[i*3+1], palette[i*3+2]
        );
        if (dist < min_dist) min_dist = dist;
    }
    
    return min_dist;
}

WASM_EXPORT unsigned int find_closest_color(
    const unsigned char* palette,
    unsigned int palette_size,
    unsigned char r, unsigned char g, unsigned char b
) {
    float min_dist = 1e30f;
    unsigned int best_idx = 0;
    
    for(unsigned int i = 0; i < palette_size; i++) {
        float dist = color_distance_perceptual(
            r, g, b,
            palette[i*3], palette[i*3+1], palette[i*3+2]
        );
        if (dist < min_dist) {
            min_dist = dist;
            best_idx = i;
        }
    }
    
    return best_idx;
}

}
