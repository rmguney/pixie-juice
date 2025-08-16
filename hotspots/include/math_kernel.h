#ifndef MATH_KERNEL_H
#define MATH_KERNEL_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// 3D vector operations
typedef struct {
    float x, y, z;
} Vec3;

typedef struct {
    float x, y, z, w;
} Vec4;

// 4x4 matrix for 3D transformations
typedef struct {
    float m[16]; // Column-major order
} Mat4;

// Quaternion for rotations
typedef struct {
    float x, y, z, w;
} Quat;

// SIMD-optimized vector operations
void vec3_add_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count);
void vec3_sub_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count);
void vec3_mul_scalar_simd(const Vec3* vectors, float scalar, Vec3* result, size_t count);
void vec3_dot_simd(const Vec3* a, const Vec3* b, float* result, size_t count);
void vec3_cross_simd(const Vec3* a, const Vec3* b, Vec3* result, size_t count);
void vec3_normalize_simd(Vec3* vectors, size_t count);

// Matrix operations
void mat4_identity(Mat4* matrix);
void mat4_multiply(const Mat4* a, const Mat4* b, Mat4* result);
void mat4_multiply_simd(const Mat4* matrices_a, const Mat4* matrices_b, Mat4* results, size_t count);
void mat4_transpose(const Mat4* matrix, Mat4* result);
void mat4_invert(const Mat4* matrix, Mat4* result);

// Transform matrices
void mat4_translation(float x, float y, float z, Mat4* result);
void mat4_rotation_x(float angle, Mat4* result);
void mat4_rotation_y(float angle, Mat4* result);
void mat4_rotation_z(float angle, Mat4* result);
void mat4_scale(float x, float y, float z, Mat4* result);
void mat4_perspective(float fov, float aspect, float near, float far, Mat4* result);
void mat4_orthographic(float left, float right, float bottom, float top, float near, float far, Mat4* result);

// Quaternion operations
void quat_identity(Quat* quat);
void quat_from_axis_angle(float x, float y, float z, float angle, Quat* result);
void quat_multiply(const Quat* a, const Quat* b, Quat* result);
void quat_normalize(Quat* quat);
void quat_slerp(const Quat* a, const Quat* b, float t, Quat* result);
void quat_to_matrix(const Quat* quat, Mat4* result);

// Batch quaternion operations for animations
void quat_slerp_batch(const Quat* start, const Quat* end, const float* t_values, Quat* results, size_t count);

// Transform points by matrix (SIMD optimized)
void transform_points_simd(const Mat4* matrix, const Vec3* points, Vec3* results, size_t count);
void transform_vectors_simd(const Mat4* matrix, const Vec3* vectors, Vec3* results, size_t count);

// Fast Fourier Transform for audio/video analysis
void fft_complex(float* real, float* imag, size_t n);
void ifft_complex(float* real, float* imag, size_t n);

// Utility functions
void compute_tangent_space(const Vec3* positions, const Vec3* normals, const float* uvs, 
                          Vec3* tangents, Vec3* bitangents, size_t vertex_count);

#ifdef __cplusplus
}
#endif

#endif // MATH_KERNEL_H
