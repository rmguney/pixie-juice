#include "mesh_attributes.h"

extern void* wasm_malloc(size_t size);
extern void wasm_free(void* ptr);

static inline float fast_abs(float x) { return __builtin_fabsf(x); }
static inline float fast_sqrt(float x) { return __builtin_sqrtf(x); }

static void set_error(MeshAttributesResult* result, const char* msg) {
    result->success = 0;
    size_t i = 0;
    for (; i < 255 && msg[i] != 0; i++) {
        result->error_message[i] = msg[i];
    }
    result->error_message[i] = 0;
}

static inline void vec3_add_inplace(float* a, const float* b) {
    a[0] += b[0];
    a[1] += b[1];
    a[2] += b[2];
}

static inline void vec3_cross(const float* a, const float* b, float* out) {
    out[0] = a[1]*b[2] - a[2]*b[1];
    out[1] = a[2]*b[0] - a[0]*b[2];
    out[2] = a[0]*b[1] - a[1]*b[0];
}

static inline float vec3_dot(const float* a, const float* b) {
    return a[0]*b[0] + a[1]*b[1] + a[2]*b[2];
}

static inline void vec3_normalize_inplace(float* v) {
    float len2 = vec3_dot(v, v);
    if (len2 <= 1e-20f) return;
    float inv = 1.0f / fast_sqrt(len2);
    v[0] *= inv;
    v[1] *= inv;
    v[2] *= inv;
}

WASM_EXPORT MeshAttributesResult compute_mesh_attributes(
    const float* vertices,
    size_t vertex_count,
    const uint32_t* indices,
    size_t index_count,
    const float* uvs,
    size_t uv_count,
    int compute_tangents
) {
    MeshAttributesResult result;
    result.normals = NULL;
    result.tangents = NULL;
    result.vertex_count = 0;
    result.success = 1;
    result.error_message[0] = 0;

    if (!vertices || vertex_count == 0) {
        set_error(&result, "Invalid vertices");
        return result;
    }
    if (!indices || index_count < 3 || (index_count % 3) != 0) {
        set_error(&result, "Invalid indices");
        return result;
    }

    float* normals = (float*)wasm_malloc(vertex_count * 3 * sizeof(float));
    if (!normals) {
        set_error(&result, "OOM normals");
        return result;
    }

    for (size_t i = 0; i < vertex_count * 3; i++) {
        normals[i] = 0.0f;
    }

    float* tangents = NULL;
    float* bitangents = NULL;
    if (compute_tangents) {
        if (!uvs || uv_count < vertex_count * 2) {
            wasm_free(normals);
            set_error(&result, "Invalid UVs for tangents");
            return result;
        }
        tangents = (float*)wasm_malloc(vertex_count * 4 * sizeof(float));
        if (!tangents) {
            wasm_free(normals);
            set_error(&result, "OOM tangents");
            return result;
        }
        bitangents = (float*)wasm_malloc(vertex_count * 3 * sizeof(float));
        if (!bitangents) {
            wasm_free(normals);
            wasm_free(tangents);
            set_error(&result, "OOM bitangents");
            return result;
        }
        for (size_t i = 0; i < vertex_count * 4; i++) {
            tangents[i] = 0.0f;
        }
        for (size_t i = 0; i < vertex_count * 3; i++) {
            bitangents[i] = 0.0f;
        }
    }

    for (size_t tri = 0; tri < index_count; tri += 3) {
        uint32_t i0 = indices[tri];
        uint32_t i1 = indices[tri + 1];
        uint32_t i2 = indices[tri + 2];
        if (i0 >= vertex_count || i1 >= vertex_count || i2 >= vertex_count) {
            wasm_free(normals);
            if (tangents) wasm_free(tangents);
            set_error(&result, "Index out of range");
            return result;
        }

        const float* v0 = &vertices[(size_t)i0 * 3];
        const float* v1 = &vertices[(size_t)i1 * 3];
        const float* v2 = &vertices[(size_t)i2 * 3];

        float e1[3] = { v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2] };
        float e2[3] = { v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2] };

        float fn[3];
        vec3_cross(e1, e2, fn);
        vec3_add_inplace(&normals[(size_t)i0 * 3], fn);
        vec3_add_inplace(&normals[(size_t)i1 * 3], fn);
        vec3_add_inplace(&normals[(size_t)i2 * 3], fn);

        if (tangents) {
            const float* uv0 = &uvs[(size_t)i0 * 2];
            const float* uv1 = &uvs[(size_t)i1 * 2];
            const float* uv2 = &uvs[(size_t)i2 * 2];

            float du1 = uv1[0] - uv0[0];
            float dv1 = uv1[1] - uv0[1];
            float du2 = uv2[0] - uv0[0];
            float dv2 = uv2[1] - uv0[1];

            float denom = du1 * dv2 - du2 * dv1;
            if (fast_abs(denom) > 1e-20f) {
                float r = 1.0f / denom;
                float t[3] = {
                    (e1[0] * dv2 - e2[0] * dv1) * r,
                    (e1[1] * dv2 - e2[1] * dv1) * r,
                    (e1[2] * dv2 - e2[2] * dv1) * r
                };
                float b[3] = {
                    (e2[0] * du1 - e1[0] * du2) * r,
                    (e2[1] * du1 - e1[1] * du2) * r,
                    (e2[2] * du1 - e1[2] * du2) * r
                };
                tangents[(size_t)i0 * 4 + 0] += t[0];
                tangents[(size_t)i0 * 4 + 1] += t[1];
                tangents[(size_t)i0 * 4 + 2] += t[2];
                tangents[(size_t)i1 * 4 + 0] += t[0];
                tangents[(size_t)i1 * 4 + 1] += t[1];
                tangents[(size_t)i1 * 4 + 2] += t[2];
                tangents[(size_t)i2 * 4 + 0] += t[0];
                tangents[(size_t)i2 * 4 + 1] += t[1];
                tangents[(size_t)i2 * 4 + 2] += t[2];

                bitangents[(size_t)i0 * 3 + 0] += b[0];
                bitangents[(size_t)i0 * 3 + 1] += b[1];
                bitangents[(size_t)i0 * 3 + 2] += b[2];
                bitangents[(size_t)i1 * 3 + 0] += b[0];
                bitangents[(size_t)i1 * 3 + 1] += b[1];
                bitangents[(size_t)i1 * 3 + 2] += b[2];
                bitangents[(size_t)i2 * 3 + 0] += b[0];
                bitangents[(size_t)i2 * 3 + 1] += b[1];
                bitangents[(size_t)i2 * 3 + 2] += b[2];
            }
        }
    }

    for (size_t i = 0; i < vertex_count; i++) {
        vec3_normalize_inplace(&normals[i * 3]);
        if (tangents) {
            float* t = &tangents[i * 4];
            float n[3] = { normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2] };
            float ndott = n[0]*t[0] + n[1]*t[1] + n[2]*t[2];
            t[0] -= n[0] * ndott;
            t[1] -= n[1] * ndott;
            t[2] -= n[2] * ndott;
            vec3_normalize_inplace(t);
            float* b = &bitangents[i * 3];
            float c[3];
            vec3_cross(n, t, c);
            float handed = vec3_dot(c, b) < 0.0f ? -1.0f : 1.0f;
            t[3] = handed;
        }
    }

    result.normals = normals;
    result.tangents = tangents;
    result.vertex_count = vertex_count;
    result.success = 1;
    result.error_message[0] = 0;

    if (bitangents) {
        wasm_free(bitangents);
    }
    return result;
}

WASM_EXPORT void free_mesh_attributes_result(MeshAttributesResult* result) {
    if (!result) return;
    if (result->normals) {
        wasm_free(result->normals);
        result->normals = NULL;
    }
    if (result->tangents) {
        wasm_free(result->tangents);
        result->tangents = NULL;
    }
    result->vertex_count = 0;
    result->success = 0;
    result->error_message[0] = 0;
}
