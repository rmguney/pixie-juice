#ifndef MATH_KERNEL_H
#define MATH_KERNEL_H

#include "memory.h"
#include "util.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    float x, y, z;
} Vec3;

typedef struct {
    float x, y, z, w;
} Vec4;

typedef struct {
    float m[16];
} Mat4;

typedef struct {
    float x, y, z, w;
} Quat;

WASM_EXPORT void vec3_add_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count);
WASM_EXPORT void vec3_sub_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count);
WASM_EXPORT void vec3_mul_scalar_simd(const Vec3* vectors, float scalar, Vec3* result, size_t count);
WASM_EXPORT void vec3_dot_simd(const Vec3* a, const Vec3* b, float* result, size_t count);
WASM_EXPORT void vec3_cross_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count);
WASM_EXPORT void vec3_normalize_simd(Vec3* vectors, size_t count);

WASM_EXPORT void mat4_identity(Mat4* matrix);
WASM_EXPORT void mat4_multiply(const Mat4* a, const Mat4* b, Mat4* result);
WASM_EXPORT void mat4_multiply_simd(const Mat4* matrices_a, const Mat4* matrices_b, Mat4* results, size_t count);
WASM_EXPORT void mat4_transpose(const Mat4* matrix, Mat4* result);
WASM_EXPORT void mat4_invert(const Mat4* matrix, Mat4* result);

WASM_EXPORT void mat4_translation(float x, float y, float z, Mat4* result);
WASM_EXPORT void mat4_rotation_x(float angle, Mat4* result);
WASM_EXPORT void mat4_rotation_y(float angle, Mat4* result);
WASM_EXPORT void mat4_rotation_z(float angle, Mat4* result);
WASM_EXPORT void mat4_scale(float x, float y, float z, Mat4* result);
WASM_EXPORT void mat4_perspective(float fov, float aspect, float near, float far, Mat4* result);
WASM_EXPORT void mat4_orthographic(float left, float right, float bottom, float top, float near, float far, Mat4* result);

WASM_EXPORT void quat_identity(Quat* quat);
WASM_EXPORT void quat_from_axis_angle(float x, float y, float z, float angle, Quat* result);
WASM_EXPORT void quat_multiply(const Quat* a, const Quat* b, Quat* result);
WASM_EXPORT void quat_normalize(Quat* quat);
WASM_EXPORT void quat_slerp(const Quat* a, const Quat* b, float t, Quat* result);
WASM_EXPORT void quat_to_matrix(const Quat* quat, Mat4* result);

WASM_EXPORT void quat_slerp_batch(const Quat* start, const Quat* end, const float* t_values, Quat* results, size_t count);

WASM_EXPORT void transform_points_simd(const Mat4* matrix, const Vec3* points, Vec3* results, size_t count);
WASM_EXPORT void transform_vectors_simd(const Mat4* matrix, const Vec3* vectors, Vec3* results, size_t count);

WASM_EXPORT void fft_complex(float* real, float* imag, size_t n);
WASM_EXPORT void ifft_complex(float* real, float* imag, size_t n);

WASM_EXPORT void compute_tangent_space(const Vec3* positions, const Vec3* normals, const float* uvs, 
                          Vec3* tangents, Vec3* bitangents, size_t vertex_count);

WASM_EXPORT void simd_vec4_add(const float* a, const float* b, float* result);
WASM_EXPORT void simd_vec4_multiply(const float* a, const float* b, float* result);
WASM_EXPORT void simd_matrix4_multiply(const float* a, const float* b, float* result);
WASM_EXPORT void simd_color_convert_batch(const float* rgb_array, float* hsv_array, size_t count);
WASM_EXPORT float vector_dot_product_simd(const float* a, const float* b, size_t count);
WASM_EXPORT void matrix_multiply_simd(const float* a, const float* b, float* result, size_t m, size_t n, size_t k);

#ifdef __cplusplus
}
#endif

#endif
