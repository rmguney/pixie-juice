#include "math_kernel.h"

static inline float fabsf_impl(float x) {
    return x < 0.0f ? -x : x;
}
#define fabsf fabsf_impl

#ifdef __wasm_simd128__
    #define SIMD_AVAILABLE 1
#else
    #define SIMD_AVAILABLE 0
#endif

#define M_PI_F 3.14159265358979323846f
#define M_PI_2_F 1.57079632679489661923f
#define M_E_F 2.71828182845904523536f
#define M_LN2_F 0.69314718055994530942f
#define M_1_PI_F 0.31830988618379067154f


static inline float fast_inv_sqrt(float x) {
    union { float f; uint32_t i; } conv = { x };
    conv.i = 0x5f3759df - (conv.i >> 1);
    float y = conv.f;
    
    y = y * (1.5f - (x * 0.5f * y * y));
    y = y * (1.5f - (x * 0.5f * y * y)); // Second iteration for better precision
    
    return y;
}

static inline float fast_sqrt(float x) {
    if (x <= 0.0f) return 0.0f;
    return x * fast_inv_sqrt(x);
}

static inline float fast_sin(float x) {
    x = x - ((int)(x * M_1_PI_F + (x >= 0 ? 0.5f : -0.5f))) * M_PI_F;
    
    if (x < 0) {
        x = -x;
        float num = 16.0f * x * (M_PI_F - x);
        float den = 5.0f * M_PI_F * M_PI_F - 4.0f * x * (M_PI_F - x);
        return -(num / den);
    } else {
        float num = 16.0f * x * (M_PI_F - x);
        float den = 5.0f * M_PI_F * M_PI_F - 4.0f * x * (M_PI_F - x);
        return num / den;
    }
}

static inline float fast_cos(float x) {
    return fast_sin(x + M_PI_2_F);
}

static inline float fast_tan(float x) {
    float s = fast_sin(x);
    float c = fast_cos(x);
    return (c == 0.0f) ? 1e30f : s / c;
}

static inline float fast_exp(float x) {
    if (x > 88.0f) return 1e30f;
    if (x < -88.0f) return 0.0f;
    
    int i = (int)x;
    float f = x - (float)i;
    
    float exp_f = 1.0f + f * (1.0f + f * (0.5f + f * (0.16666667f + f * 0.04166667f)));
    
    float exp_i = 1.0f;
    float base = M_E_F;
    int exp = i < 0 ? -i : i;
    
    while (exp > 0) {
        if (exp & 1) exp_i *= base;
        base *= base;
        exp >>= 1;
    }
    
    return i < 0 ? exp_f / exp_i : exp_f * exp_i;
}

static inline float fast_log(float x) {
    if (x <= 0.0f) return -1e30f;
    if (x == 1.0f) return 0.0f;
    
    union { float f; uint32_t i; } u = { x };
    float log2_x = (float)((int)(u.i >> 23) - 127);
    u.i = (u.i & 0x007FFFFF) | 0x3F800000;
    float y = u.f - 1.0f;
    
    float y2 = y * y;
    float y3 = y2 * y;
    float result = y - 0.5f * y2 + 0.33333333f * y3 - 0.25f * y2 * y2;
    
    return result + log2_x * M_LN2_F;
}

static inline float fast_pow(float x, float y) {
    if (x <= 0.0f) return 0.0f;
    if (y == 0.0f) return 1.0f;
    if (y == 1.0f) return x;
    return fast_exp(y * fast_log(x));
}

void vec3_normalize(float* v) {
    if (!v) return;
    
    float len_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if (len_sq > 1e-10f) {
        float inv_len = fast_inv_sqrt(len_sq);
        v[0] *= inv_len;
        v[1] *= inv_len;
        v[2] *= inv_len;
    }
}

float vec3_dot(const float* a, const float* b) {
    if (!a || !b) return 0.0f;
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

void vec3_cross(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    
    result[0] = a[1] * b[2] - a[2] * b[1];
    result[1] = a[2] * b[0] - a[0] * b[2];
    result[2] = a[0] * b[1] - a[1] * b[0];
}

void vec3_add(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    
    result[0] = a[0] + b[0];
    result[1] = a[1] + b[1];
    result[2] = a[2] + b[2];
}

void vec3_subtract(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    
    result[0] = a[0] - b[0];
    result[1] = a[1] - b[1];
    result[2] = a[2] - b[2];
}

void vec3_scale(const float* v, float scale, float* result) {
    if (!v || !result) return;
    
    result[0] = v[0] * scale;
    result[1] = v[1] * scale;
    result[2] = v[2] * scale;
}

float vec3_length(const float* v) {
    if (!v) return 0.0f;
    return fast_sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
}

void matrix4_identity(float* matrix) {
    if (!matrix) return;
    
    for (int i = 0; i < 16; i++) {
        matrix[i] = 0.0f;
    }
    matrix[0] = matrix[5] = matrix[10] = matrix[15] = 1.0f;
}

void matrix4_multiply(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            result[i * 4 + j] = 0.0f;
            for (int k = 0; k < 4; k++) {
                result[i * 4 + j] += a[i * 4 + k] * b[k * 4 + j];
            }
        }
    }
}

void matrix4_transpose(const float* matrix, float* result) {
    if (!matrix || !result) return;
    
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            result[j * 4 + i] = matrix[i * 4 + j];
        }
    }
}

void rgb_to_hsv(float r, float g, float b, float* h, float* s, float* v) {
    if (!h || !s || !v) return;
    
    float max_val = (r > g) ? ((r > b) ? r : b) : ((g > b) ? g : b);
    float min_val = (r < g) ? ((r < b) ? r : b) : ((g < b) ? g : b);
    float delta = max_val - min_val;
    
    *v = max_val;
    *s = (max_val > 1e-6f) ? (delta / max_val) : 0.0f;
    
    if (delta < 1e-6f) {
        *h = 0.0f;
    } else if (max_val == r) {
        *h = 60.0f * ((g - b) / delta);
        if (*h < 0.0f) *h += 360.0f;
    } else if (max_val == g) {
        *h = 60.0f * ((b - r) / delta) + 120.0f;
    } else {
        *h = 60.0f * ((r - g) / delta) + 240.0f;
    }
}

void hsv_to_rgb(float h, float s, float v, float* r, float* g, float* b) {
    if (!r || !g || !b) return;
    
    if (s < 1e-6f) {
        *r = *g = *b = v;
        return;
    }
    
    h = h - ((int)(h / 360.0f)) * 360.0f;
    if (h < 0.0f) h += 360.0f;
    
    float c = v * s;
    float x = c * (1.0f - fabsf(((h / 60.0f) - 2.0f * (int)(h / 120.0f)) - 1.0f));
    float m = v - c;
    
    if (h < 60.0f) {
        *r = c; *g = x; *b = 0.0f;
    } else if (h < 120.0f) {
        *r = x; *g = c; *b = 0.0f;
    } else if (h < 180.0f) {
        *r = 0.0f; *g = c; *b = x;
    } else if (h < 240.0f) {
        *r = 0.0f; *g = x; *b = c;
    } else if (h < 300.0f) {
        *r = x; *g = 0.0f; *b = c;
    } else {
        *r = c; *g = 0.0f; *b = x;
    }
    
    *r += m; *g += m; *b += m;
}

void rgb_to_lab(float r, float g, float b, float* l, float* a_out, float* b_out) {
    if (!l || !a_out || !b_out) return;
    
    r = (r > 0.04045f) ? fast_pow((r + 0.055f) / 1.055f, 2.4f) : r / 12.92f;
    g = (g > 0.04045f) ? fast_pow((g + 0.055f) / 1.055f, 2.4f) : g / 12.92f;
    b = (b > 0.04045f) ? fast_pow((b + 0.055f) / 1.055f, 2.4f) : b / 12.92f;
    
    float x = r * 0.4124564f + g * 0.3575761f + b * 0.1804375f;
    float y = r * 0.2126729f + g * 0.7151522f + b * 0.0721750f;
    float z = r * 0.0193339f + g * 0.1191920f + b * 0.9503041f;
    
    x /= 0.95047f;
    y /= 1.00000f;
    z /= 1.08883f;
    
    float fx = (x > 0.008856f) ? fast_pow(x, 1.0f/3.0f) : (7.787f * x + 16.0f/116.0f);
    float fy = (y > 0.008856f) ? fast_pow(y, 1.0f/3.0f) : (7.787f * y + 16.0f/116.0f);
    float fz = (z > 0.008856f) ? fast_pow(z, 1.0f/3.0f) : (7.787f * z + 16.0f/116.0f);
    
    *l = 116.0f * fy - 16.0f;
    *a_out = 500.0f * (fx - fy);
    *b_out = 200.0f * (fy - fz);
}

#if SIMD_AVAILABLE

WASM_EXPORT void simd_vec4_add(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    
    v128_t va = wasm_v128_load(a);
    v128_t vb = wasm_v128_load(b);
    v128_t vr = wasm_f32x4_add(va, vb);
    wasm_v128_store(result, vr);
}

WASM_EXPORT void simd_vec4_multiply(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    
    v128_t va = wasm_v128_load(a);
    v128_t vb = wasm_v128_load(b);
    v128_t vr = wasm_f32x4_mul(va, vb);
    wasm_v128_store(result, vr);
}

WASM_EXPORT void simd_matrix4_multiply(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    
    for (int i = 0; i < 4; i++) {
        v128_t row_result = wasm_f32x4_const(0, 0, 0, 0);
        
        for (int j = 0; j < 4; j++) {
            v128_t a_element = wasm_f32x4_splat(a[i * 4 + j]);
            v128_t b_row = wasm_v128_load(&b[j * 4]);
            row_result = wasm_f32x4_add(row_result, wasm_f32x4_mul(a_element, b_row));
        }
        
        wasm_v128_store(&result[i * 4], row_result);
    }
}

WASM_EXPORT void simd_color_convert_batch(const float* rgb_array, float* hsv_array, size_t count) {
    if (!rgb_array || !hsv_array) return;
    
    size_t simd_count = (count / 4) * 4;
    
    for (size_t i = 0; i < simd_count; i += 4) {
        v128_t r_vals = wasm_f32x4_make(rgb_array[i*3], rgb_array[(i+1)*3], 
                                        rgb_array[(i+2)*3], rgb_array[(i+3)*3]);
        v128_t g_vals = wasm_f32x4_make(rgb_array[i*3+1], rgb_array[(i+1)*3+1], 
                                        rgb_array[(i+2)*3+1], rgb_array[(i+3)*3+1]);
        v128_t b_vals = wasm_f32x4_make(rgb_array[i*3+2], rgb_array[(i+1)*3+2], 
                                        rgb_array[(i+2)*3+2], rgb_array[(i+3)*3+2]);
        
        v128_t max_val = wasm_f32x4_max(wasm_f32x4_max(r_vals, g_vals), b_vals);
        v128_t min_val = wasm_f32x4_min(wasm_f32x4_min(r_vals, g_vals), b_vals);
        v128_t delta = wasm_f32x4_sub(max_val, min_val);
        
        wasm_v128_store(&hsv_array[i*3+2], max_val);
        
        v128_t saturation = wasm_f32x4_div(delta, max_val);
        wasm_v128_store(&hsv_array[i*3+1], saturation);
        
        for (size_t j = 0; j < 4 && (i + j) < count; j++) {
            float h, s, v;
            rgb_to_hsv(rgb_array[(i+j)*3], rgb_array[(i+j)*3+1], rgb_array[(i+j)*3+2], &h, &s, &v);
            hsv_array[(i+j)*3] = h;
        }
    }
    
    for (size_t i = simd_count; i < count; i++) {
        float h, s, v;
        rgb_to_hsv(rgb_array[i*3], rgb_array[i*3+1], rgb_array[i*3+2], &h, &s, &v);
        hsv_array[i*3] = h;
        hsv_array[i*3+1] = s;
        hsv_array[i*3+2] = v;
    }
}

#else

WASM_EXPORT void simd_vec4_add(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    for (int i = 0; i < 4; i++) result[i] = a[i] + b[i];
}

WASM_EXPORT void simd_vec4_multiply(const float* a, const float* b, float* result) {
    if (!a || !b || !result) return;
    for (int i = 0; i < 4; i++) result[i] = a[i] * b[i];
}

WASM_EXPORT void simd_matrix4_multiply(const float* a, const float* b, float* result) {
    matrix4_multiply(a, b, result);
}

WASM_EXPORT void simd_color_convert_batch(const float* rgb_array, float* hsv_array, size_t count) {
    if (!rgb_array || !hsv_array) return;
    for (size_t i = 0; i < count; i++) {
        float h, s, v;
        rgb_to_hsv(rgb_array[i*3], rgb_array[i*3+1], rgb_array[i*3+2], &h, &s, &v);
        hsv_array[i*3] = h;
        hsv_array[i*3+1] = s;
        hsv_array[i*3+2] = v;
    }
}

#endif

float clamp(float value, float min_val, float max_val) {
    if (value < min_val) return min_val;
    if (value > max_val) return max_val;
    return value;
}

float lerp(float a, float b, float t) {
    return a + t * (b - a);
}

float smoothstep(float edge0, float edge1, float x) {
    float t = clamp((x - edge0) / (edge1 - edge0), 0.0f, 1.0f);
    return t * t * (3.0f - 2.0f * t);
}

int32_t fast_round(float x) {
    return (int32_t)(x + (x >= 0 ? 0.5f : -0.5f));
}

uint32_t next_power_of_2(uint32_t x) {
    x--;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    return x + 1;
}

float vector_dot_product_simd(const float* a, const float* b, size_t count) {
    float sum = 0.0f;
    for (size_t i = 0; i < count; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}

void matrix_multiply_simd(const float* a, const float* b, float* result, size_t m, size_t n, size_t k) {
    for (size_t r = 0; r < m; r++) {
        for (size_t c = 0; c < k; c++) {
            float sum = 0.0f;
            for (size_t i = 0; i < n; i++) {
                sum += a[r * n + i] * b[i * k + c];
            }
            result[r * k + c] = sum;
        }
    }
}
