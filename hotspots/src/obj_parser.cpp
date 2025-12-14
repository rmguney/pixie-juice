#include "obj_parser.h"

static inline bool is_space(uint8_t c) {
    return c == ' ' || c == '\t' || c == '\r';
}

static inline bool is_newline(uint8_t c) {
    return c == '\n';
}

static inline bool is_digit(uint8_t c) {
    return c >= '0' && c <= '9';
}

static inline const uint8_t* skip_spaces(const uint8_t* p, const uint8_t* end) {
    while (p < end && is_space(*p)) {
        p++;
    }
    return p;
}

static inline const uint8_t* skip_to_eol(const uint8_t* p, const uint8_t* end) {
    while (p < end && !is_newline(*p)) {
        p++;
    }
    if (p < end && is_newline(*p)) {
        p++;
    }
    return p;
}

static inline float pow10_i32(int32_t e) {
    float v = 1.0f;
    if (e > 0) {
        for (int32_t i = 0; i < e; i++) {
            v *= 10.0f;
        }
    } else if (e < 0) {
        for (int32_t i = 0; i < -e; i++) {
            v *= 0.1f;
        }
    }
    return v;
}

static bool parse_i32(const uint8_t*& p, const uint8_t* end, int32_t& out) {
    p = skip_spaces(p, end);
    if (p >= end) return false;

    int32_t sign = 1;
    if (*p == '-') {
        sign = -1;
        p++;
    } else if (*p == '+') {
        p++;
    }

    if (p >= end || !is_digit(*p)) return false;

    int32_t v = 0;
    while (p < end && is_digit(*p)) {
        v = v * 10 + (int32_t)(*p - '0');
        p++;
    }

    out = v * sign;
    return true;
}

static bool parse_f32(const uint8_t*& p, const uint8_t* end, float& out) {
    p = skip_spaces(p, end);
    if (p >= end) return false;

    int32_t sign = 1;
    if (*p == '-') {
        sign = -1;
        p++;
    } else if (*p == '+') {
        p++;
    }

    uint32_t int_part = 0;
    uint32_t frac_part = 0;
    uint32_t frac_div = 1;

    bool has_digits = false;

    while (p < end && is_digit(*p)) {
        has_digits = true;
        int_part = int_part * 10u + (uint32_t)(*p - '0');
        p++;
    }

    if (p < end && *p == '.') {
        p++;
        while (p < end && is_digit(*p)) {
            has_digits = true;
            frac_part = frac_part * 10u + (uint32_t)(*p - '0');
            frac_div *= 10u;
            p++;
        }
    }

    if (!has_digits) {
        return false;
    }

    float v = (float)int_part;
    if (frac_div != 1u) {
        v += ((float)frac_part) / (float)frac_div;
    }

    if (p < end && (*p == 'e' || *p == 'E')) {
        p++;
        int32_t exp = 0;
        if (!parse_i32(p, end, exp)) {
            return false;
        }
        v *= pow10_i32(exp);
    }

    out = v * (float)sign;
    return true;
}

static inline bool starts_with(const uint8_t* p, const uint8_t* end, const char* lit) {
    const uint8_t* q = (const uint8_t*)lit;
    while (*q) {
        if (p >= end) return false;
        if (*p != *q) return false;
        p++;
        q++;
    }
    return true;
}

static void set_error(ObjParseResult* r, const char* msg) {
    if (!r) return;
    r->success = 0;
    size_t i = 0;
    while (msg[i] && i + 1 < sizeof(r->error_message)) {
        r->error_message[i] = msg[i];
        i++;
    }
    r->error_message[i] = 0;
}

extern "C" {

WASM_EXPORT ObjParseResult* obj_parse_to_mesh(const uint8_t* data, size_t data_len) {
    ObjParseResult* result = (ObjParseResult*)wasm_malloc(sizeof(ObjParseResult));
    if (!result) return nullptr;

    memset(result, 0, sizeof(ObjParseResult));
    result->success = 0;

    if (!data || data_len == 0) {
        set_error(result, "OBJ parse: empty input");
        return result;
    }

    const uint8_t* p = data;
    const uint8_t* end = data + data_len;

    size_t v_count = 0;
    size_t vn_count = 0;
    size_t vt_count = 0;
    size_t tri_index_count = 0;

    bool saw_object = false;
    const uint8_t* object_name_ptr = nullptr;
    size_t object_name_len = 0;

    while (p < end) {
        const uint8_t* line = p;
        p = skip_spaces(p, end);

        if (p >= end) break;
        if (*p == '#') {
            p = skip_to_eol(p, end);
            continue;
        }
        if (is_newline(*p)) {
            p++;
            continue;
        }

        if (starts_with(p, end, "o")) {
            const uint8_t* t = p + 1;
            if (t < end && is_space(*t)) {
                if (!saw_object) {
                    t = skip_spaces(t, end);
                    object_name_ptr = t;
                    while (t < end && !is_space(*t) && !is_newline(*t)) {
                        t++;
                    }
                    object_name_len = (size_t)(t - object_name_ptr);
                    saw_object = true;
                }
            }
            p = skip_to_eol(line, end);
            continue;
        }

        if (starts_with(p, end, "vn")) {
            const uint8_t* t = p + 2;
            if (t < end && is_space(*t)) {
                vn_count++;
            }
            p = skip_to_eol(line, end);
            continue;
        }

        if (starts_with(p, end, "vt")) {
            const uint8_t* t = p + 2;
            if (t < end && is_space(*t)) {
                vt_count++;
            }
            p = skip_to_eol(line, end);
            continue;
        }

        if (starts_with(p, end, "v")) {
            const uint8_t* t = p + 1;
            if (t < end && is_space(*t)) {
                v_count++;
            }
            p = skip_to_eol(line, end);
            continue;
        }

        if (starts_with(p, end, "f")) {
            const uint8_t* t = p + 1;
            if (t < end && is_space(*t)) {
                t = skip_spaces(t, end);
                size_t verts_in_face = 0;
                while (t < end && !is_newline(*t)) {
                    t = skip_spaces(t, end);
                    if (t >= end || is_newline(*t)) break;

                    int32_t idx_val = 0;
                    const uint8_t* idx_start = t;
                    if (!parse_i32(t, end, idx_val)) {
                        t = skip_to_eol(line, end);
                        verts_in_face = 0;
                        break;
                    }

                    (void)idx_start;

                    while (t < end && *t != ' ' && *t != '\t' && *t != '\r' && *t != '\n') {
                        if (*t == ' ') break;
                        if (*t == '\t') break;
                        if (*t == '\r') break;
                        if (*t == '\n') break;
                        if (*t == '/') {
                            while (t < end && *t != ' ' && *t != '\t' && *t != '\r' && *t != '\n') {
                                t++;
                            }
                            break;
                        }
                        t++;
                    }

                    verts_in_face++;
                }

                if (verts_in_face >= 3) {
                    tri_index_count += (verts_in_face - 2) * 3;
                }
            }
            p = skip_to_eol(line, end);
            continue;
        }

        p = skip_to_eol(line, end);
    }

    if (v_count == 0 || tri_index_count == 0) {
        set_error(result, "OBJ parse: no vertices or faces");
        result->success = 1;
        return result;
    }

    result->vertices = (float*)wasm_malloc(v_count * 3 * sizeof(float));
    result->normals = vn_count ? (float*)wasm_malloc(vn_count * 3 * sizeof(float)) : nullptr;
    result->texcoords = vt_count ? (float*)wasm_malloc(vt_count * 2 * sizeof(float)) : nullptr;
    result->indices = (uint32_t*)wasm_malloc(tri_index_count * sizeof(uint32_t));

    result->vertex_count = v_count;
    result->normal_count = vn_count;
    result->texcoord_count = vt_count;
    result->index_count = tri_index_count;

    if (!result->vertices || !result->indices || (vn_count && !result->normals) || (vt_count && !result->texcoords)) {
        set_error(result, "OBJ parse: allocation failed");
        return result;
    }

    if (saw_object && object_name_ptr && object_name_len) {
        result->object_name = (char*)wasm_malloc(object_name_len + 1);
        if (result->object_name) {
            for (size_t i = 0; i < object_name_len; i++) {
                result->object_name[i] = (char)object_name_ptr[i];
            }
            result->object_name[object_name_len] = 0;
            result->object_name_len = object_name_len;
        }
    }

    size_t v_written = 0;
    size_t vn_written = 0;
    size_t vt_written = 0;
    size_t idx_written = 0;

    p = data;

    while (p < end) {
        const uint8_t* line = p;
        p = skip_spaces(p, end);

        if (p >= end) break;
        if (*p == '#') {
            p = skip_to_eol(p, end);
            continue;
        }
        if (is_newline(*p)) {
            p++;
            continue;
        }

        if (starts_with(p, end, "vn")) {
            const uint8_t* t = p + 2;
            if (t < end && is_space(*t)) {
                t = skip_spaces(t, end);
                float x = 0.0f, y = 0.0f, z = 0.0f;
                if (parse_f32(t, end, x) && parse_f32(t, end, y) && parse_f32(t, end, z)) {
                    if (vn_written < vn_count) {
                        result->normals[vn_written * 3 + 0] = x;
                        result->normals[vn_written * 3 + 1] = y;
                        result->normals[vn_written * 3 + 2] = z;
                        vn_written++;
                    }
                }
            }
            p = skip_to_eol(line, end);
            continue;
        }

        if (starts_with(p, end, "vt")) {
            const uint8_t* t = p + 2;
            if (t < end && is_space(*t)) {
                t = skip_spaces(t, end);
                float u = 0.0f, v = 0.0f;
                if (parse_f32(t, end, u) && parse_f32(t, end, v)) {
                    if (vt_written < vt_count) {
                        result->texcoords[vt_written * 2 + 0] = u;
                        result->texcoords[vt_written * 2 + 1] = v;
                        vt_written++;
                    }
                }
            }
            p = skip_to_eol(line, end);
            continue;
        }

        if (starts_with(p, end, "v")) {
            const uint8_t* t = p + 1;
            if (t < end && is_space(*t)) {
                t = skip_spaces(t, end);
                float x = 0.0f, y = 0.0f, z = 0.0f;
                if (parse_f32(t, end, x) && parse_f32(t, end, y) && parse_f32(t, end, z)) {
                    if (v_written < v_count) {
                        result->vertices[v_written * 3 + 0] = x;
                        result->vertices[v_written * 3 + 1] = y;
                        result->vertices[v_written * 3 + 2] = z;
                        v_written++;
                    }
                }
            }
            p = skip_to_eol(line, end);
            continue;
        }

        if (starts_with(p, end, "f")) {
            const uint8_t* t = p + 1;
            if (t < end && is_space(*t)) {
                t = skip_spaces(t, end);

                uint32_t face_tmp[64];
                size_t face_n = 0;

                while (t < end && !is_newline(*t)) {
                    t = skip_spaces(t, end);
                    if (t >= end || is_newline(*t)) break;

                    int32_t idx_val = 0;
                    if (!parse_i32(t, end, idx_val)) {
                        face_n = 0;
                        break;
                    }

                    while (t < end && *t != ' ' && *t != '\t' && *t != '\r' && *t != '\n') {
                        if (*t == '/') {
                            while (t < end && *t != ' ' && *t != '\t' && *t != '\r' && *t != '\n') {
                                t++;
                            }
                            break;
                        }
                        t++;
                    }

                    if (face_n < 64) {
                        int32_t resolved = idx_val;
                        if (resolved < 0) {
                            resolved = (int32_t)v_count + resolved + 1;
                        }
                        if (resolved > 0) {
                            face_tmp[face_n++] = (uint32_t)(resolved - 1);
                        }
                    }
                }

                if (face_n >= 3) {
                    for (size_t k = 2; k < face_n; k++) {
                        if (idx_written + 3 <= tri_index_count) {
                            result->indices[idx_written++] = face_tmp[0];
                            result->indices[idx_written++] = face_tmp[k - 1];
                            result->indices[idx_written++] = face_tmp[k];
                        }
                    }
                }
            }
            p = skip_to_eol(line, end);
            continue;
        }

        p = skip_to_eol(line, end);
    }

    result->vertex_count = v_written;
    result->normal_count = vn_written;
    result->texcoord_count = vt_written;
    result->index_count = idx_written;

    result->success = 1;
    return result;
}

WASM_EXPORT void free_obj_parse_result(ObjParseResult* result) {
    if (!result) return;
    if (result->vertices) wasm_free(result->vertices);
    if (result->normals) wasm_free(result->normals);
    if (result->texcoords) wasm_free(result->texcoords);
    if (result->indices) wasm_free(result->indices);
    if (result->object_name) wasm_free(result->object_name);
    wasm_free(result);
}

}
