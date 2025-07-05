#include "math_kernel.h"

// Only include standard headers for native builds
#ifndef __wasm__
    #include <math.h>
    #include <string.h>
#endif

// Define M_PI if not available
#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

// WASM-compatible implementations
#ifdef __wasm__
    // Math functions for WASM - float versions
    static inline float sinf(float x) {
        // Simple Taylor series approximation for WASM
        float result = x;
        float term = x;
        for (int i = 1; i <= 5; i++) {
            term *= -x * x / ((2 * i) * (2 * i + 1));
            result += term;
        }
        return result;
    }
    
    static inline float cosf(float x) {
        // Simple Taylor series approximation for WASM
        float result = 1.0f;
        float term = 1.0f;
        for (int i = 1; i <= 5; i++) {
            term *= -x * x / ((2 * i - 1) * (2 * i));
            result += term;
        }
        return result;
    }
    
    static inline float sqrtf(float x) {
        if (x < 0.0f) return 0.0f;
        if (x == 0.0f) return 0.0f;
        float guess = x * 0.5f;
        for (int i = 0; i < 10; i++) {
            guess = (guess + x / guess) * 0.5f;
        }
        return guess;
    }
    
    static inline float tanf(float x) {
        // tan(x) = sin(x) / cos(x)
        return sinf(x) / cosf(x);
    }
    
    static inline float fabsf(float x) { return x < 0.0f ? -x : x; }
    
    static inline float acosf(float x) {
        // Clamp x to [-1, 1] to avoid NaN
        if (x <= -1.0f) return M_PI;
        if (x >= 1.0f) return 0.0f;
        
        // Use Taylor series approximation: arccos(x) = π/2 - arcsin(x)
        // arcsin(x) ≈ x + x³/6 + 3x⁵/40 + ...
        float x2 = x * x;
        float arcsin_x = x + (x * x2) / 6.0f + (3.0f * x * x2 * x2) / 40.0f;
        return M_PI * 0.5f - arcsin_x;
    }
    
    // Math functions for WASM - double versions (reuse existing)
    static inline double sin(double x) {
        // Simple Taylor series approximation for WASM
        double result = x;
        double term = x;
        for (int i = 1; i <= 5; i++) {
            term *= -x * x / ((2 * i) * (2 * i + 1));
            result += term;
        }
        return result;
    }
    
    static inline double cos(double x) {
        // Simple Taylor series approximation for WASM
        double result = 1.0;
        double term = 1.0;
        for (int i = 1; i <= 5; i++) {
            term *= -x * x / ((2 * i - 1) * (2 * i));
            result += term;
        }
        return result;
    }
    
    static inline double sqrt(double x) {
        if (x < 0.0) return 0.0;
        if (x == 0.0) return 0.0;
        double guess = x * 0.5;
        for (int i = 0; i < 10; i++) {
            guess = (guess + x / guess) * 0.5;
        }
        return guess;
    }
    
    static inline double fabs(double x) { return x < 0.0 ? -x : x; }
    
    // Memory operations for WASM
    static void* memset_wasm(void* s, int c, size_t n) {
        char* p = (char*)s;
        for (size_t i = 0; i < n; i++) {
            p[i] = (char)c;
        }
        return s;
    }
    
    #define memset memset_wasm
#endif

// WASM SIMD support (if available)
#ifdef __wasm__
    #ifdef __wasm_simd128__
        #include <wasm_simd128.h>
        #define WASM_SIMD_AVAILABLE 1
        
        // WASM SIMD wrapper types and functions to match x86 interface
        typedef v128_t __m128;
        #define _mm_set_ps(w, z, y, x) wasm_f32x4_make(x, y, z, w)
        #define _mm_set1_ps(a) wasm_f32x4_splat(a)
        #define _mm_add_ps(a, b) wasm_f32x4_add(a, b)
        #define _mm_sub_ps(a, b) wasm_f32x4_sub(a, b)
        #define _mm_mul_ps(a, b) wasm_f32x4_mul(a, b)
        #define _mm_div_ps(a, b) wasm_f32x4_div(a, b)
        
        static inline void _mm_store_ps(float* mem_addr, __m128 a) {
            wasm_v128_store(mem_addr, a);
        }
        
        static inline __m128 _mm_load_ps(const float* mem_addr) {
            return wasm_v128_load(mem_addr);
        }
        
        static inline float _mm_cvtss_f32(__m128 a) {
            return wasm_f32x4_extract_lane(a, 0);
        }
        
        static inline void _mm_store_ss(float* mem_addr, __m128 a) {
            *mem_addr = wasm_f32x4_extract_lane(a, 0);
        }
        
        // Dot product emulation for WASM
        static inline __m128 _mm_dp_ps(__m128 a, __m128 b, unsigned int mask) {
            // For simplicity, just do a basic dot product and splat
            __m128 mul = wasm_f32x4_mul(a, b);
            float result = wasm_f32x4_extract_lane(mul, 0) + 
                          wasm_f32x4_extract_lane(mul, 1) + 
                          wasm_f32x4_extract_lane(mul, 2) + 
                          wasm_f32x4_extract_lane(mul, 3);
            return wasm_f32x4_splat(result);
        }
        
        // Reciprocal square root approximation for WASM
        static inline __m128 _mm_rsqrt_ps(__m128 a) {
            // Simple reciprocal square root approximation
            __m128 sqrt_a = wasm_f32x4_sqrt(a);
            __m128 one = wasm_f32x4_splat(1.0f);
            return wasm_f32x4_div(one, sqrt_a);
        }
        
        // Shuffle emulation for WASM
        static inline __m128 _mm_shuffle_ps(__m128 a, __m128 b, unsigned int mask) {
            // WASM SIMD requires compile-time constants for lane indices
            // We'll need to handle this with a runtime switch for common shuffle patterns
            switch (mask) {
                case 0x00:  // [0,0,0,0]
                    return wasm_f32x4_make(
                        wasm_f32x4_extract_lane(a, 0),
                        wasm_f32x4_extract_lane(a, 0),
                        wasm_f32x4_extract_lane(b, 0),
                        wasm_f32x4_extract_lane(b, 0)
                    );
                case 0x55:  // [1,1,1,1]
                    return wasm_f32x4_make(
                        wasm_f32x4_extract_lane(a, 1),
                        wasm_f32x4_extract_lane(a, 1),
                        wasm_f32x4_extract_lane(b, 1),
                        wasm_f32x4_extract_lane(b, 1)
                    );
                case 0xAA:  // [2,2,2,2]
                    return wasm_f32x4_make(
                        wasm_f32x4_extract_lane(a, 2),
                        wasm_f32x4_extract_lane(a, 2),
                        wasm_f32x4_extract_lane(b, 2),
                        wasm_f32x4_extract_lane(b, 2)
                    );
                case 0xFF:  // [3,3,3,3]
                    return wasm_f32x4_make(
                        wasm_f32x4_extract_lane(a, 3),
                        wasm_f32x4_extract_lane(a, 3),
                        wasm_f32x4_extract_lane(b, 3),
                        wasm_f32x4_extract_lane(b, 3)
                    );
                default:
                    // For other patterns, fall back to element 0
                    return wasm_f32x4_make(
                        wasm_f32x4_extract_lane(a, 0),
                        wasm_f32x4_extract_lane(a, 0),
                        wasm_f32x4_extract_lane(b, 0),
                        wasm_f32x4_extract_lane(b, 0)
                    );
            }
        }
        
    #else
        #define WASM_SIMD_AVAILABLE 0
        // Fallback: disable SIMD for WASM without SIMD support
        #define X86_SIMD_AVAILABLE 0
    #endif
#endif

// x86 SIMD support for native builds
#if !defined(__wasm__) && (defined(_MSC_VER) || defined(__GNUC__))
    #ifdef _MSC_VER
        #include <intrin.h>
    #else
        #include <x86intrin.h>
    #endif
    #define X86_SIMD_AVAILABLE 1
#else
    #ifndef WASM_SIMD_AVAILABLE
        #define X86_SIMD_AVAILABLE 0
    #endif
#endif

// Enable SIMD if either x86 or WASM SIMD is available
#if defined(X86_SIMD_AVAILABLE) && X86_SIMD_AVAILABLE
    #define SIMD_AVAILABLE 1
#elif defined(WASM_SIMD_AVAILABLE) && WASM_SIMD_AVAILABLE
    #define SIMD_AVAILABLE 1
#else
    #define SIMD_AVAILABLE 0
#endif

// Feature detection
static int has_simd = -1;
static int has_sse = 0;

static void detect_simd_features() {
    if (has_simd == -1) {
#ifdef __wasm__
        // Check for WASM SIMD support
        #ifdef __wasm_simd128__
            has_simd = 1;
            has_sse = 1; // WASM SIMD is roughly equivalent to SSE
        #else
            has_simd = 0;
            has_sse = 0;
        #endif
#else
        // Detect x86 SIMD features
        int cpuinfo[4];
        #ifdef _MSC_VER
            __cpuid(cpuinfo, 1);
            has_sse = (cpuinfo[3] & (1 << 25)) != 0; // SSE support
            has_simd = has_sse;
        #else
            __builtin_cpu_init();
            has_sse = __builtin_cpu_supports("sse") ? 1 : 0;
            has_simd = has_sse;
        #endif
#endif
    }
}

// SSE-optimized vector operations (4 vectors at a time)
void vec3_add_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count) {
    detect_simd_features();
    
    if (has_sse && count >= 4) {
        size_t simd_count = count & ~3; // Round down to multiple of 4
        
        for (size_t i = 0; i < simd_count; i += 4) {
            // Load 4 vectors worth of x components
            __m128 ax = _mm_set_ps(a[i+3].x, a[i+2].x, a[i+1].x, a[i].x);
            __m128 bx = _mm_set_ps(b[i+3].x, b[i+2].x, b[i+1].x, b[i].x);
            __m128 rx = _mm_add_ps(ax, bx);
            
            // Load 4 vectors worth of y components
            __m128 ay = _mm_set_ps(a[i+3].y, a[i+2].y, a[i+1].y, a[i].y);
            __m128 by = _mm_set_ps(b[i+3].y, b[i+2].y, b[i+1].y, b[i].y);
            __m128 ry = _mm_add_ps(ay, by);
            
            // Load 4 vectors worth of z components
            __m128 az = _mm_set_ps(a[i+3].z, a[i+2].z, a[i+1].z, a[i].z);
            __m128 bz = _mm_set_ps(b[i+3].z, b[i+2].z, b[i+1].z, b[i].z);
            __m128 rz = _mm_add_ps(az, bz);
            
            // Store results
            float temp[4];
            _mm_store_ps(temp, rx);
            result[i].x = temp[0]; result[i+1].x = temp[1]; result[i+2].x = temp[2]; result[i+3].x = temp[3];
            
            _mm_store_ps(temp, ry);
            result[i].y = temp[0]; result[i+1].y = temp[1]; result[i+2].y = temp[2]; result[i+3].y = temp[3];
            
            _mm_store_ps(temp, rz);
            result[i].z = temp[0]; result[i+1].z = temp[1]; result[i+2].z = temp[2]; result[i+3].z = temp[3];
        }
        
        // Handle remaining elements
        for (size_t i = simd_count; i < count; i++) {
            result[i].x = a[i].x + b[i].x;
            result[i].y = a[i].y + b[i].y;
            result[i].z = a[i].z + b[i].z;
        }
    } else {
        // Fallback to scalar implementation
        for (size_t i = 0; i < count; i++) {
            result[i].x = a[i].x + b[i].x;
            result[i].y = a[i].y + b[i].y;
            result[i].z = a[i].z + b[i].z;
        }
    }
}

void vec3_sub_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count) {
    detect_simd_features();
    
    if (has_sse && count >= 4) {
        size_t simd_count = count & ~3;
        
        for (size_t i = 0; i < simd_count; i += 4) {
            __m128 ax = _mm_set_ps(a[i+3].x, a[i+2].x, a[i+1].x, a[i].x);
            __m128 bx = _mm_set_ps(b[i+3].x, b[i+2].x, b[i+1].x, b[i].x);
            __m128 rx = _mm_sub_ps(ax, bx);
            
            __m128 ay = _mm_set_ps(a[i+3].y, a[i+2].y, a[i+1].y, a[i].y);
            __m128 by = _mm_set_ps(b[i+3].y, b[i+2].y, b[i+1].y, b[i].y);
            __m128 ry = _mm_sub_ps(ay, by);
            
            __m128 az = _mm_set_ps(a[i+3].z, a[i+2].z, a[i+1].z, a[i].z);
            __m128 bz = _mm_set_ps(b[i+3].z, b[i+2].z, b[i+1].z, b[i].z);
            __m128 rz = _mm_sub_ps(az, bz);
            
            float temp[4];
            _mm_store_ps(temp, rx);
            result[i].x = temp[0]; result[i+1].x = temp[1]; result[i+2].x = temp[2]; result[i+3].x = temp[3];
            
            _mm_store_ps(temp, ry);
            result[i].y = temp[0]; result[i+1].y = temp[1]; result[i+2].y = temp[2]; result[i+3].y = temp[3];
            
            _mm_store_ps(temp, rz);
            result[i].z = temp[0]; result[i+1].z = temp[1]; result[i+2].z = temp[2]; result[i+3].z = temp[3];
        }
        
        for (size_t i = simd_count; i < count; i++) {
            result[i].x = a[i].x - b[i].x;
            result[i].y = a[i].y - b[i].y;
            result[i].z = a[i].z - b[i].z;
        }
    } else {
        for (size_t i = 0; i < count; i++) {
            result[i].x = a[i].x - b[i].x;
            result[i].y = a[i].y - b[i].y;
            result[i].z = a[i].z - b[i].z;
        }
    }
}

void vec3_mul_scalar_simd(const Vec3* vectors, float scalar, Vec3* result, size_t count) {
    detect_simd_features();
    
    if (has_sse && count >= 4) {
        size_t simd_count = count & ~3;
        __m128 scalar_vec = _mm_set1_ps(scalar);
        
        for (size_t i = 0; i < simd_count; i += 4) {
            __m128 vx = _mm_set_ps(vectors[i+3].x, vectors[i+2].x, vectors[i+1].x, vectors[i].x);
            __m128 vy = _mm_set_ps(vectors[i+3].y, vectors[i+2].y, vectors[i+1].y, vectors[i].y);
            __m128 vz = _mm_set_ps(vectors[i+3].z, vectors[i+2].z, vectors[i+1].z, vectors[i].z);
            
            __m128 rx = _mm_mul_ps(vx, scalar_vec);
            __m128 ry = _mm_mul_ps(vy, scalar_vec);
            __m128 rz = _mm_mul_ps(vz, scalar_vec);
            
            float temp[4];
            _mm_store_ps(temp, rx);
            result[i].x = temp[0]; result[i+1].x = temp[1]; result[i+2].x = temp[2]; result[i+3].x = temp[3];
            
            _mm_store_ps(temp, ry);
            result[i].y = temp[0]; result[i+1].y = temp[1]; result[i+2].y = temp[2]; result[i+3].y = temp[3];
            
            _mm_store_ps(temp, rz);
            result[i].z = temp[0]; result[i+1].z = temp[1]; result[i+2].z = temp[2]; result[i+3].z = temp[3];
        }
        
        for (size_t i = simd_count; i < count; i++) {
            result[i].x = vectors[i].x * scalar;
            result[i].y = vectors[i].y * scalar;
            result[i].z = vectors[i].z * scalar;
        }
    } else {
        for (size_t i = 0; i < count; i++) {
            result[i].x = vectors[i].x * scalar;
            result[i].y = vectors[i].y * scalar;
            result[i].z = vectors[i].z * scalar;
        }
    }
}

void vec3_dot_simd(const Vec3* a, const Vec3* b, float* result, size_t count) {
    detect_simd_features();
    
    if (has_sse && count >= 4) {
        size_t simd_count = count & ~3;
        
        for (size_t i = 0; i < simd_count; i += 4) {
            __m128 ax = _mm_set_ps(a[i+3].x, a[i+2].x, a[i+1].x, a[i].x);
            __m128 bx = _mm_set_ps(b[i+3].x, b[i+2].x, b[i+1].x, b[i].x);
            __m128 ay = _mm_set_ps(a[i+3].y, a[i+2].y, a[i+1].y, a[i].y);
            __m128 by = _mm_set_ps(b[i+3].y, b[i+2].y, b[i+1].y, b[i].y);
            __m128 az = _mm_set_ps(a[i+3].z, a[i+2].z, a[i+1].z, a[i].z);
            __m128 bz = _mm_set_ps(b[i+3].z, b[i+2].z, b[i+1].z, b[i].z);
            
            __m128 dot = _mm_add_ps(_mm_add_ps(_mm_mul_ps(ax, bx), _mm_mul_ps(ay, by)), _mm_mul_ps(az, bz));
            
            float temp[4];
            _mm_store_ps(temp, dot);
            result[i] = temp[0]; result[i+1] = temp[1]; result[i+2] = temp[2]; result[i+3] = temp[3];
        }
        
        for (size_t i = simd_count; i < count; i++) {
            result[i] = a[i].x * b[i].x + a[i].y * b[i].y + a[i].z * b[i].z;
        }
    } else {
        for (size_t i = 0; i < count; i++) {
            result[i] = a[i].x * b[i].x + a[i].y * b[i].y + a[i].z * b[i].z;
        }
    }
}

void vec3_cross_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count) {
    detect_simd_features();
    
    if (has_sse && count >= 4) {
        size_t simd_count = count & ~3;
        
        for (size_t i = 0; i < simd_count; i += 4) {
            __m128 ax = _mm_set_ps(a[i+3].x, a[i+2].x, a[i+1].x, a[i].x);
            __m128 ay = _mm_set_ps(a[i+3].y, a[i+2].y, a[i+1].y, a[i].y);
            __m128 az = _mm_set_ps(a[i+3].z, a[i+2].z, a[i+1].z, a[i].z);
            __m128 bx = _mm_set_ps(b[i+3].x, b[i+2].x, b[i+1].x, b[i].x);
            __m128 by = _mm_set_ps(b[i+3].y, b[i+2].y, b[i+1].y, b[i].y);
            __m128 bz = _mm_set_ps(b[i+3].z, b[i+2].z, b[i+1].z, b[i].z);
            
            // Cross product: (ay*bz - az*by, az*bx - ax*bz, ax*by - ay*bx)
            __m128 rx = _mm_sub_ps(_mm_mul_ps(ay, bz), _mm_mul_ps(az, by));
            __m128 ry = _mm_sub_ps(_mm_mul_ps(az, bx), _mm_mul_ps(ax, bz));
            __m128 rz = _mm_sub_ps(_mm_mul_ps(ax, by), _mm_mul_ps(ay, bx));
            
            float temp[4];
            _mm_store_ps(temp, rx);
            result[i].x = temp[0]; result[i+1].x = temp[1]; result[i+2].x = temp[2]; result[i+3].x = temp[3];
            
            _mm_store_ps(temp, ry);
            result[i].y = temp[0]; result[i+1].y = temp[1]; result[i+2].y = temp[2]; result[i+3].y = temp[3];
            
            _mm_store_ps(temp, rz);
            result[i].z = temp[0]; result[i+1].z = temp[1]; result[i+2].z = temp[2]; result[i+3].z = temp[3];
        }
        
        for (size_t i = simd_count; i < count; i++) {
            result[i].x = a[i].y * b[i].z - a[i].z * b[i].y;
            result[i].y = a[i].z * b[i].x - a[i].x * b[i].z;
            result[i].z = a[i].x * b[i].y - a[i].y * b[i].x;
        }
    } else {
        for (size_t i = 0; i < count; i++) {
            result[i].x = a[i].y * b[i].z - a[i].z * b[i].y;
            result[i].y = a[i].z * b[i].x - a[i].x * b[i].z;
            result[i].z = a[i].x * b[i].y - a[i].y * b[i].x;
        }
    }
}

void vec3_normalize_simd(Vec3* vectors, size_t count) {
    detect_simd_features();
    
    if (has_sse) {
        // Process 4 vectors at once when possible
        size_t simd_count = (count / 4) * 4;
        
        for (size_t i = 0; i < simd_count; i += 4) {
            // Load 4 vectors (12 floats) - we'll process them individually but with SIMD ops
            for (size_t j = 0; j < 4 && (i + j) < count; j++) {
                __m128 vec = _mm_set_ps(0.0f, vectors[i+j].z, vectors[i+j].y, vectors[i+j].x);
                
                // Calculate length squared using dot product
                __m128 len_sq = _mm_dp_ps(vec, vec, 0x77);  // dot product with mask
                
                // Use fast reciprocal square root
                __m128 rsqrt = _mm_rsqrt_ps(len_sq);
                
                // One Newton-Raphson iteration for better precision
                // rsqrt = rsqrt * (1.5 - 0.5 * len_sq * rsqrt * rsqrt)
                __m128 half = _mm_set1_ps(0.5f);
                __m128 one_half = _mm_set1_ps(1.5f);
                __m128 rsqrt_sq = _mm_mul_ps(rsqrt, rsqrt);
                __m128 correction = _mm_sub_ps(one_half, _mm_mul_ps(half, _mm_mul_ps(len_sq, rsqrt_sq)));
                rsqrt = _mm_mul_ps(rsqrt, correction);
                
                // Normalize the vector
                __m128 normalized = _mm_mul_ps(vec, rsqrt);
                
                // Store back (check for zero length)
                float len_check;
                _mm_store_ss(&len_check, len_sq);
                if (len_check > 1e-8f) {
                    vectors[i+j].x = _mm_cvtss_f32(normalized);
                    vectors[i+j].y = _mm_cvtss_f32(_mm_shuffle_ps(normalized, normalized, 0x55));
                    vectors[i+j].z = _mm_cvtss_f32(_mm_shuffle_ps(normalized, normalized, 0xAA));
                }
            }
        }
        
        // Handle remaining vectors
        for (size_t i = simd_count; i < count; i++) {
            float len_sq = vectors[i].x * vectors[i].x + 
                          vectors[i].y * vectors[i].y + 
                          vectors[i].z * vectors[i].z;
            if (len_sq > 1e-8f) {
                float inv_len = 1.0f / sqrtf(len_sq);
                vectors[i].x *= inv_len;
                vectors[i].y *= inv_len;
                vectors[i].z *= inv_len;
            }
        }
    } else {
        // Fallback for non-SIMD
        for (size_t i = 0; i < count; i++) {
            float len_sq = vectors[i].x * vectors[i].x + 
                          vectors[i].y * vectors[i].y + 
                          vectors[i].z * vectors[i].z;
            if (len_sq > 1e-8f) {
                float inv_len = 1.0f / sqrtf(len_sq);
                vectors[i].x *= inv_len;
                vectors[i].y *= inv_len;
                vectors[i].z *= inv_len;
            }
        }
    }
}

void mat4_identity(Mat4* matrix) {
    memset(matrix->m, 0, sizeof(matrix->m));
    matrix->m[0] = matrix->m[5] = matrix->m[10] = matrix->m[15] = 1.0f;
}

void mat4_multiply(const Mat4* a, const Mat4* b, Mat4* result) {
    detect_simd_features();
    
    if (has_sse) {
        // Load matrix b columns
        __m128 b_col0 = _mm_load_ps(&b->m[0]);
        __m128 b_col1 = _mm_load_ps(&b->m[4]);
        __m128 b_col2 = _mm_load_ps(&b->m[8]);
        __m128 b_col3 = _mm_load_ps(&b->m[12]);
        
        for (int i = 0; i < 4; i++) {
            __m128 a_row = _mm_set_ps(a->m[i*4+3], a->m[i*4+2], a->m[i*4+1], a->m[i*4+0]);
            
            __m128 result_row = _mm_add_ps(
                _mm_add_ps(
                    _mm_mul_ps(_mm_shuffle_ps(a_row, a_row, 0x00), b_col0),
                    _mm_mul_ps(_mm_shuffle_ps(a_row, a_row, 0x55), b_col1)
                ),
                _mm_add_ps(
                    _mm_mul_ps(_mm_shuffle_ps(a_row, a_row, 0xAA), b_col2),
                    _mm_mul_ps(_mm_shuffle_ps(a_row, a_row, 0xFF), b_col3)
                )
            );
            
            _mm_store_ps(&result->m[i*4], result_row);
        }
    } else {
        // Fallback scalar implementation
        for (int i = 0; i < 4; i++) {
            for (int j = 0; j < 4; j++) {
                result->m[i*4 + j] = 0.0f;
                for (int k = 0; k < 4; k++) {
                    result->m[i*4 + j] += a->m[i*4 + k] * b->m[k*4 + j];
                }
            }
        }
    }
}

void mat4_multiply_simd(const Mat4* matrices_a, const Mat4* matrices_b, Mat4* results, size_t count) {
    // Use the optimized single matrix multiply for each pair
    for (size_t i = 0; i < count; i++) {
        mat4_multiply(&matrices_a[i], &matrices_b[i], &results[i]);
    }
}

void mat4_transpose(const Mat4* matrix, Mat4* result) {
    for (int i = 0; i < 4; i++) {
        for (int j = 0; j < 4; j++) {
            result->m[i*4 + j] = matrix->m[j*4 + i];
        }
    }
}

void mat4_invert(const Mat4* matrix, Mat4* result) {
    // Optimized 4x4 matrix inversion using cofactor expansion
    const float* m = matrix->m;
    float* inv = result->m;
    
    // Calculate determinants of 2x2 submatrices
    float s0 = m[0] * m[5] - m[4] * m[1];
    float s1 = m[0] * m[6] - m[4] * m[2];
    float s2 = m[0] * m[7] - m[4] * m[3];
    float s3 = m[1] * m[6] - m[5] * m[2];
    float s4 = m[1] * m[7] - m[5] * m[3];
    float s5 = m[2] * m[7] - m[6] * m[3];
    
    float c5 = m[10] * m[15] - m[14] * m[11];
    float c4 = m[9] * m[15] - m[13] * m[11];
    float c3 = m[9] * m[14] - m[13] * m[10];
    float c2 = m[8] * m[15] - m[12] * m[11];
    float c1 = m[8] * m[14] - m[12] * m[10];
    float c0 = m[8] * m[13] - m[12] * m[9];
    
    // Calculate determinant
    float det = s0 * c5 - s1 * c4 + s2 * c3 + s3 * c2 - s4 * c1 + s5 * c0;
    
    if (fabsf(det) < 1e-8f) {
        // Matrix is singular, return identity
        mat4_identity(result);
        return;
    }
    
    float inv_det = 1.0f / det;
    
    // Calculate inverse matrix elements
    inv[0] = (m[5] * c5 - m[6] * c4 + m[7] * c3) * inv_det;
    inv[1] = (-m[1] * c5 + m[2] * c4 - m[3] * c3) * inv_det;
    inv[2] = (m[13] * s5 - m[14] * s4 + m[15] * s3) * inv_det;
    inv[3] = (-m[9] * s5 + m[10] * s4 - m[11] * s3) * inv_det;
    
    inv[4] = (-m[4] * c5 + m[6] * c2 - m[7] * c1) * inv_det;
    inv[5] = (m[0] * c5 - m[2] * c2 + m[3] * c1) * inv_det;
    inv[6] = (-m[12] * s5 + m[14] * s2 - m[15] * s1) * inv_det;
    inv[7] = (m[8] * s5 - m[10] * s2 + m[11] * s1) * inv_det;
    
    inv[8] = (m[4] * c4 - m[5] * c2 + m[7] * c0) * inv_det;
    inv[9] = (-m[0] * c4 + m[1] * c2 - m[3] * c0) * inv_det;
    inv[10] = (m[12] * s4 - m[13] * s2 + m[15] * s0) * inv_det;
    inv[11] = (-m[8] * s4 + m[9] * s2 - m[11] * s0) * inv_det;
    
    inv[12] = (-m[4] * c3 + m[5] * c1 - m[6] * c0) * inv_det;
    inv[13] = (m[0] * c3 - m[1] * c1 + m[2] * c0) * inv_det;
    inv[14] = (-m[12] * s3 + m[13] * s1 - m[14] * s0) * inv_det;
    inv[15] = (m[8] * s3 - m[9] * s1 + m[10] * s0) * inv_det;
}

void mat4_translation(float x, float y, float z, Mat4* result) {
    mat4_identity(result);
    result->m[12] = x;
    result->m[13] = y;
    result->m[14] = z;
}

void mat4_rotation_x(float angle, Mat4* result) {
    mat4_identity(result);
    float c = cosf(angle);
    float s = sinf(angle);
    result->m[5] = c; result->m[6] = -s;
    result->m[9] = s; result->m[10] = c;
}

void mat4_rotation_y(float angle, Mat4* result) {
    mat4_identity(result);
    float c = cosf(angle);
    float s = sinf(angle);
    result->m[0] = c; result->m[2] = s;
    result->m[8] = -s; result->m[10] = c;
}

void mat4_rotation_z(float angle, Mat4* result) {
    mat4_identity(result);
    float c = cosf(angle);
    float s = sinf(angle);
    result->m[0] = c; result->m[1] = -s;
    result->m[4] = s; result->m[5] = c;
}

void mat4_scale(float x, float y, float z, Mat4* result) {
    mat4_identity(result);
    result->m[0] = x;
    result->m[5] = y;
    result->m[10] = z;
}

void mat4_perspective(float fov, float aspect, float near, float far, Mat4* result) {
    memset(result->m, 0, sizeof(result->m));
    float f = 1.0f / tanf(fov * 0.5f);
    result->m[0] = f / aspect;
    result->m[5] = f;
    result->m[10] = (far + near) / (near - far);
    result->m[11] = -1.0f;
    result->m[14] = (2.0f * far * near) / (near - far);
}

void mat4_orthographic(float left, float right, float bottom, float top, float near, float far, Mat4* result) {
    memset(result->m, 0, sizeof(result->m));
    result->m[0] = 2.0f / (right - left);
    result->m[5] = 2.0f / (top - bottom);
    result->m[10] = -2.0f / (far - near);
    result->m[12] = -(right + left) / (right - left);
    result->m[13] = -(top + bottom) / (top - bottom);
    result->m[14] = -(far + near) / (far - near);
    result->m[15] = 1.0f;
}

void quat_identity(Quat* quat) {
    quat->x = quat->y = quat->z = 0.0f;
    quat->w = 1.0f;
}

void quat_from_axis_angle(float x, float y, float z, float angle, Quat* result) {
    float half_angle = angle * 0.5f;
    float s = sinf(half_angle);
    result->x = x * s;
    result->y = y * s;
    result->z = z * s;
    result->w = cosf(half_angle);
}

void quat_multiply(const Quat* a, const Quat* b, Quat* result) {
    result->x = a->w * b->x + a->x * b->w + a->y * b->z - a->z * b->y;
    result->y = a->w * b->y - a->x * b->z + a->y * b->w + a->z * b->x;
    result->z = a->w * b->z + a->x * b->y - a->y * b->x + a->z * b->w;
    result->w = a->w * b->w - a->x * b->x - a->y * b->y - a->z * b->z;
}

void quat_normalize(Quat* quat) {
    float len = sqrtf(quat->x * quat->x + quat->y * quat->y + 
                     quat->z * quat->z + quat->w * quat->w);
    if (len > 0.0f) {
        float inv_len = 1.0f / len;
        quat->x *= inv_len;
        quat->y *= inv_len;
        quat->z *= inv_len;
        quat->w *= inv_len;
    }
}

void quat_slerp(const Quat* a, const Quat* b, float t, Quat* result) {
    // Spherical linear interpolation for quaternions
    float dot = a->x * b->x + a->y * b->y + a->z * b->z + a->w * b->w;
    
    // Use the shorter path (flip b if dot is negative)
    Quat b_corrected = *b;
    if (dot < 0.0f) {
        b_corrected.x = -b->x;
        b_corrected.y = -b->y;
        b_corrected.z = -b->z;
        b_corrected.w = -b->w;
        dot = -dot;
    }
    
    if (dot > 0.9995f) {
        // Quaternions are very close, use linear interpolation
        result->x = a->x + t * (b_corrected.x - a->x);
        result->y = a->y + t * (b_corrected.y - a->y);
        result->z = a->z + t * (b_corrected.z - a->z);
        result->w = a->w + t * (b_corrected.w - a->w);
        quat_normalize(result);
        return;
    }
    
    // Use spherical interpolation
    float theta = acosf(dot);
    float sin_theta = sinf(theta);
    float inv_sin_theta = 1.0f / sin_theta;
    
    float scale_a = sinf((1.0f - t) * theta) * inv_sin_theta;
    float scale_b = sinf(t * theta) * inv_sin_theta;
    
    result->x = scale_a * a->x + scale_b * b_corrected.x;
    result->y = scale_a * a->y + scale_b * b_corrected.y;
    result->z = scale_a * a->z + scale_b * b_corrected.z;
    result->w = scale_a * a->w + scale_b * b_corrected.w;
}

void quat_to_matrix(const Quat* quat, Mat4* result) {
    // Convert quaternion to 4x4 rotation matrix
    float x2 = quat->x * quat->x;
    float y2 = quat->y * quat->y;
    float z2 = quat->z * quat->z;
    float xy = quat->x * quat->y;
    float xz = quat->x * quat->z;
    float yz = quat->y * quat->z;
    float wx = quat->w * quat->x;
    float wy = quat->w * quat->y;
    float wz = quat->w * quat->z;
    
    result->m[0] = 1.0f - 2.0f * (y2 + z2);
    result->m[1] = 2.0f * (xy + wz);
    result->m[2] = 2.0f * (xz - wy);
    result->m[3] = 0.0f;
    
    result->m[4] = 2.0f * (xy - wz);
    result->m[5] = 1.0f - 2.0f * (x2 + z2);
    result->m[6] = 2.0f * (yz + wx);
    result->m[7] = 0.0f;
    
    result->m[8] = 2.0f * (xz + wy);
    result->m[9] = 2.0f * (yz - wx);
    result->m[10] = 1.0f - 2.0f * (x2 + y2);
    result->m[11] = 0.0f;
    
    result->m[12] = 0.0f;
    result->m[13] = 0.0f;
    result->m[14] = 0.0f;
    result->m[15] = 1.0f;
}

void quat_slerp_batch(const Quat* start, const Quat* end, const float* t_values, Quat* results, size_t count) {
    detect_simd_features();
    
    if (has_sse && count >= 4) {
        // Process 4 quaternions at once using SIMD
        size_t simd_count = (count / 4) * 4;
        
        for (size_t i = 0; i < simd_count; i += 4) {
            // Load quaternions and t values
            __m128 t_vec = _mm_load_ps(&t_values[i]);
            
            for (size_t j = 0; j < 4; j++) {
                // For now, fall back to scalar for complex SLERP logic
                // Full SIMD SLERP would require significant code
                float t = t_values[i + j];
                quat_slerp(&start[i + j], &end[i + j], t, &results[i + j]);
            }
        }
        
        // Handle remaining quaternions
        for (size_t i = simd_count; i < count; i++) {
            quat_slerp(&start[i], &end[i], t_values[i], &results[i]);
        }
    } else {
        // Scalar fallback
        for (size_t i = 0; i < count; i++) {
            quat_slerp(&start[i], &end[i], t_values[i], &results[i]);
        }
    }
}

void transform_points_simd(const Mat4* matrix, const Vec3* points, Vec3* results, size_t count) {
    detect_simd_features();
    
    if (has_sse) {
        // Load matrix rows for SIMD operations
        __m128 row0 = _mm_load_ps(&matrix->m[0]);
        __m128 row1 = _mm_load_ps(&matrix->m[4]);
        __m128 row2 = _mm_load_ps(&matrix->m[8]);
        __m128 row3 = _mm_load_ps(&matrix->m[12]);
        
        for (size_t i = 0; i < count; i++) {
            // Load point as (x, y, z, 1)
            __m128 point = _mm_set_ps(1.0f, points[i].z, points[i].y, points[i].x);
            
            // Matrix-vector multiplication
            __m128 result_x = _mm_dp_ps(row0, point, 0xF1);
            __m128 result_y = _mm_dp_ps(row1, point, 0xF2);
            __m128 result_z = _mm_dp_ps(row2, point, 0xF4);
            
            // Store results
            _mm_store_ss(&results[i].x, result_x);
            _mm_store_ss(&results[i].y, result_y);
            _mm_store_ss(&results[i].z, result_z);
        }
    } else {
        // Scalar fallback
        for (size_t i = 0; i < count; i++) {
            const Vec3* p = &points[i];
            results[i].x = matrix->m[0] * p->x + matrix->m[4] * p->y + matrix->m[8] * p->z + matrix->m[12];
            results[i].y = matrix->m[1] * p->x + matrix->m[5] * p->y + matrix->m[9] * p->z + matrix->m[13];
            results[i].z = matrix->m[2] * p->x + matrix->m[6] * p->y + matrix->m[10] * p->z + matrix->m[14];
        }
    }
}

void transform_vectors_simd(const Mat4* matrix, const Vec3* vectors, Vec3* results, size_t count) {
    detect_simd_features();
    
    if (has_sse) {
        // Load matrix rows for SIMD operations (no translation)
        __m128 row0 = _mm_set_ps(0.0f, matrix->m[8], matrix->m[4], matrix->m[0]);
        __m128 row1 = _mm_set_ps(0.0f, matrix->m[9], matrix->m[5], matrix->m[1]);
        __m128 row2 = _mm_set_ps(0.0f, matrix->m[10], matrix->m[6], matrix->m[2]);
        
        for (size_t i = 0; i < count; i++) {
            // Load vector as (x, y, z, 0)
            __m128 vector = _mm_set_ps(0.0f, vectors[i].z, vectors[i].y, vectors[i].x);
            
            // Matrix-vector multiplication (no translation)
            __m128 result_x = _mm_dp_ps(row0, vector, 0x71);
            __m128 result_y = _mm_dp_ps(row1, vector, 0x72);
            __m128 result_z = _mm_dp_ps(row2, vector, 0x74);
            
            // Store results
            _mm_store_ss(&results[i].x, result_x);
            _mm_store_ss(&results[i].y, result_y);
            _mm_store_ss(&results[i].z, result_z);
        }
    } else {
        // Scalar fallback
        for (size_t i = 0; i < count; i++) {
            const Vec3* v = &vectors[i];
            results[i].x = matrix->m[0] * v->x + matrix->m[4] * v->y + matrix->m[8] * v->z;
            results[i].y = matrix->m[1] * v->x + matrix->m[5] * v->y + matrix->m[9] * v->z;
            results[i].z = matrix->m[2] * v->x + matrix->m[6] * v->y + matrix->m[10] * v->z;
        }
    }
}

void fft_complex(float* real, float* imag, size_t n) {
    // Cooley-Tukey FFT algorithm implementation
    if (n <= 1) return;
    
    // Bit-reversal permutation
    for (size_t i = 1, j = 0; i < n; i++) {
        size_t bit = n >> 1;
        for (; j & bit; bit >>= 1) {
            j ^= bit;
        }
        j ^= bit;
        
        if (i < j) {
            // Swap real parts
            float temp = real[i];
            real[i] = real[j];
            real[j] = temp;
            
            // Swap imaginary parts
            temp = imag[i];
            imag[i] = imag[j];
            imag[j] = temp;
        }
    }
    
    // Cooley-Tukey FFT
    for (size_t len = 2; len <= n; len <<= 1) {
        float angle = -2.0f * M_PI / len;
        float wlen_real = cosf(angle);
        float wlen_imag = sinf(angle);
        
        for (size_t i = 0; i < n; i += len) {
            float w_real = 1.0f;
            float w_imag = 0.0f;
            
            for (size_t j = 0; j < len / 2; j++) {
                size_t u = i + j;
                size_t v = i + j + len / 2;
                
                float u_real = real[u];
                float u_imag = imag[u];
                float v_real = real[v];
                float v_imag = imag[v];
                
                // Complex multiplication: w * v
                float temp_real = w_real * v_real - w_imag * v_imag;
                float temp_imag = w_real * v_imag + w_imag * v_real;
                
                real[u] = u_real + temp_real;
                imag[u] = u_imag + temp_imag;
                real[v] = u_real - temp_real;
                imag[v] = u_imag - temp_imag;
                
                // Update w for next iteration
                float next_w_real = w_real * wlen_real - w_imag * wlen_imag;
                float next_w_imag = w_real * wlen_imag + w_imag * wlen_real;
                w_real = next_w_real;
                w_imag = next_w_imag;
            }
        }
    }
}

void ifft_complex(float* real, float* imag, size_t n) {
    // Inverse FFT: conjugate input, apply FFT, conjugate output, scale by 1/n
    
    // Conjugate input (negate imaginary parts)
    for (size_t i = 0; i < n; i++) {
        imag[i] = -imag[i];
    }
    
    // Apply forward FFT
    fft_complex(real, imag, n);
    
    // Conjugate output and scale
    float scale = 1.0f / (float)n;
    for (size_t i = 0; i < n; i++) {
        real[i] *= scale;
        imag[i] *= -scale; // Conjugate and scale
    }
}

float fast_sqrt(float x) {
    // Fast square root using bit manipulation and Newton-Raphson
    if (x <= 0.0f) return 0.0f;
    
    // Initial approximation using bit manipulation
    union { float f; uint32_t i; } conv;
    conv.f = x;
    conv.i = 0x5f3759df - (conv.i >> 1); // Magic number for inverse sqrt
    float y = conv.f;
    
    // Newton-Raphson iterations for better precision
    y = y * (1.5f - (0.5f * x * y * y));
    y = y * (1.5f - (0.5f * x * y * y)); // Second iteration for accuracy
    
    // Convert inverse sqrt to sqrt
    return x * y;
}

float fast_inv_sqrt(float x) {
    // Famous Quake III inverse square root
    if (x <= 0.0f) return 0.0f;
    
    union { float f; uint32_t i; } conv;
    conv.f = x;
    float x2 = x * 0.5f;
    conv.i = 0x5f3759df - (conv.i >> 1); // What the fuck?
    float y = conv.f;
    
    // Newton-Raphson iterations
    y = y * (1.5f - (x2 * y * y)); // 1st iteration
    y = y * (1.5f - (x2 * y * y)); // 2nd iteration, this can be removed
    
    return y;
}

void compute_tangent_space(const Vec3* positions, const Vec3* normals, const float* uvs, 
                          Vec3* tangents, Vec3* bitangents, size_t vertex_count) {
    // Compute tangent space for normal mapping
    for (size_t i = 0; i < vertex_count; i += 3) {
        if (i + 2 >= vertex_count) break;
        
        // Get triangle vertices
        const Vec3* v0 = &positions[i];
        const Vec3* v1 = &positions[i + 1];
        const Vec3* v2 = &positions[i + 2];
        
        // Get UV coordinates
        float u0 = uvs[i * 2], v0_uv = uvs[i * 2 + 1];
        float u1 = uvs[(i + 1) * 2], v1_uv = uvs[(i + 1) * 2 + 1];
        float u2 = uvs[(i + 2) * 2], v2_uv = uvs[(i + 2) * 2 + 1];
        
        // Calculate edge vectors
        Vec3 edge1 = {v1->x - v0->x, v1->y - v0->y, v1->z - v0->z};
        Vec3 edge2 = {v2->x - v0->x, v2->y - v0->y, v2->z - v0->z};
        
        // Calculate UV deltas
        float delta_u1 = u1 - u0;
        float delta_v1 = v1_uv - v0_uv;
        float delta_u2 = u2 - u0;
        float delta_v2 = v2_uv - v0_uv;
        
        // Calculate tangent and bitangent
        float determinant = delta_u1 * delta_v2 - delta_u2 * delta_v1;
        if (fabsf(determinant) < 1e-8f) {
            // Degenerate case, use arbitrary orthogonal vectors
            for (size_t j = 0; j < 3; j++) {
                tangents[i + j] = (Vec3){1.0f, 0.0f, 0.0f};
                bitangents[i + j] = (Vec3){0.0f, 1.0f, 0.0f};
            }
            continue;
        }
        
        float inv_det = 1.0f / determinant;
        
        Vec3 tangent = {
            inv_det * (delta_v2 * edge1.x - delta_v1 * edge2.x),
            inv_det * (delta_v2 * edge1.y - delta_v1 * edge2.y),
            inv_det * (delta_v2 * edge1.z - delta_v1 * edge2.z)
        };
        
        Vec3 bitangent = {
            inv_det * (-delta_u2 * edge1.x + delta_u1 * edge2.x),
            inv_det * (-delta_u2 * edge1.y + delta_u1 * edge2.y),
            inv_det * (-delta_u2 * edge1.z + delta_u1 * edge2.z)
        };
        
        // Assign to all three vertices of the triangle
        for (size_t j = 0; j < 3; j++) {
            // Gram-Schmidt orthogonalize tangent against normal
            const Vec3* normal = &normals[i + j];
            float dot = tangent.x * normal->x + tangent.y * normal->y + tangent.z * normal->z;
            
            Vec3 orthogonal_tangent = {
                tangent.x - dot * normal->x,
                tangent.y - dot * normal->y,
                tangent.z - dot * normal->z
            };
            
            // Normalize
            float len = sqrtf(orthogonal_tangent.x * orthogonal_tangent.x + 
                             orthogonal_tangent.y * orthogonal_tangent.y + 
                             orthogonal_tangent.z * orthogonal_tangent.z);
            if (len > 1e-8f) {
                float inv_len = 1.0f / len;
                orthogonal_tangent.x *= inv_len;
                orthogonal_tangent.y *= inv_len;
                orthogonal_tangent.z *= inv_len;
            }
            
            tangents[i + j] = orthogonal_tangent;
            
            // Compute bitangent as cross product of normal and tangent
            bitangents[i + j] = (Vec3){
                normal->y * orthogonal_tangent.z - normal->z * orthogonal_tangent.y,
                normal->z * orthogonal_tangent.x - normal->x * orthogonal_tangent.z,
                normal->x * orthogonal_tangent.y - normal->y * orthogonal_tangent.x
            };
        }
    }
}
