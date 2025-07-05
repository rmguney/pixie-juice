#include "math_kernel.h"
#include <math.h>
#include <string.h>

#ifdef _MSC_VER
#include <intrin.h>
#else
#include <x86intrin.h>
#endif

// Feature detection
static int has_sse = -1;
static int has_avx = -1;

static void detect_simd_features() {
    if (has_sse == -1) {
        int cpuinfo[4];
#ifdef _MSC_VER
        __cpuid(cpuinfo, 1);
#else
        __builtin_cpu_init();
        cpuinfo[3] = __builtin_cpu_supports("sse") ? (1 << 25) : 0;
        cpuinfo[2] = __builtin_cpu_supports("avx") ? (1 << 28) : 0;
#endif
        has_sse = (cpuinfo[3] & (1 << 25)) != 0;
        has_avx = (cpuinfo[2] & (1 << 28)) != 0;
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
    // TODO: Use SSE/AVX for SIMD acceleration with rsqrt
    for (size_t i = 0; i < count; i++) {
        float len = sqrtf(vectors[i].x * vectors[i].x + 
                         vectors[i].y * vectors[i].y + 
                         vectors[i].z * vectors[i].z);
        if (len > 0.0f) {
            float inv_len = 1.0f / len;
            vectors[i].x *= inv_len;
            vectors[i].y *= inv_len;
            vectors[i].z *= inv_len;
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
    // TODO: Implement optimized 4x4 matrix inversion
    // For now, set to identity as placeholder
    mat4_identity(result);
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
    // TODO: Implement proper spherical linear interpolation
    // Placeholder linear interpolation
    result->x = a->x + t * (b->x - a->x);
    result->y = a->y + t * (b->y - a->y);
    result->z = a->z + t * (b->z - a->z);
    result->w = a->w + t * (b->w - a->w);
    quat_normalize(result);
}

void quat_to_matrix(const Quat* quat, Mat4* result) {
    // TODO: Implement quaternion to matrix conversion
    mat4_identity(result);
}

void quat_slerp_batch(const Quat* start, const Quat* end, const float* t_values, Quat* results, size_t count) {
    // TODO: Implement SIMD batch quaternion interpolation
    for (size_t i = 0; i < count; i++) {
        quat_slerp(&start[i], &end[i], t_values[i], &results[i]);
    }
}

void transform_points_simd(const Mat4* matrix, const Vec3* points, Vec3* results, size_t count) {
    // TODO: Implement SIMD matrix-vector multiplication
    for (size_t i = 0; i < count; i++) {
        const Vec3* p = &points[i];
        results[i].x = matrix->m[0] * p->x + matrix->m[4] * p->y + matrix->m[8] * p->z + matrix->m[12];
        results[i].y = matrix->m[1] * p->x + matrix->m[5] * p->y + matrix->m[9] * p->z + matrix->m[13];
        results[i].z = matrix->m[2] * p->x + matrix->m[6] * p->y + matrix->m[10] * p->z + matrix->m[14];
    }
}

void transform_vectors_simd(const Mat4* matrix, const Vec3* vectors, Vec3* results, size_t count) {
    // TODO: Implement SIMD matrix-vector multiplication (no translation)
    for (size_t i = 0; i < count; i++) {
        const Vec3* v = &vectors[i];
        results[i].x = matrix->m[0] * v->x + matrix->m[4] * v->y + matrix->m[8] * v->z;
        results[i].y = matrix->m[1] * v->x + matrix->m[5] * v->y + matrix->m[9] * v->z;
        results[i].z = matrix->m[2] * v->x + matrix->m[6] * v->y + matrix->m[10] * v->z;
    }
}

void fft_complex(float* real, float* imag, size_t n) {
    // TODO: Implement optimized FFT using Cooley-Tukey algorithm
    (void)real; (void)imag; (void)n;
    // Placeholder implementation
}

void ifft_complex(float* real, float* imag, size_t n) {
    // TODO: Implement optimized inverse FFT
    (void)real; (void)imag; (void)n;
    // Placeholder implementation
}

float fast_sqrt(float x) {
    // TODO: Implement fast approximation using bit manipulation
    return sqrtf(x);
}

float fast_inv_sqrt(float x) {
    // TODO: Implement famous Quake III inverse square root
    return 1.0f / sqrtf(x);
}

void compute_tangent_space(const Vec3* positions, const Vec3* normals, const float* uvs, 
                          Vec3* tangents, Vec3* bitangents, size_t vertex_count) {
    // TODO: Implement tangent space computation for normal mapping
    (void)positions; (void)normals; (void)uvs; (void)tangents; (void)bitangents; (void)vertex_count;
    // Placeholder implementation
}
