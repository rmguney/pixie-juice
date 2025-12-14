#include "vertex_cache.h"

extern void* wasm_malloc(size_t size);
extern void wasm_free(void* ptr);

typedef struct {
    float score;
    int cache_pos;
    uint32_t active_tris;
    uint32_t* tris;
    uint32_t tri_count;
} VCVertex;

typedef struct {
    uint32_t v[3];
    float score;
    int emitted;
} VCTriangle;

typedef struct {
    uint32_t* heap;
    uint32_t* pos;
    float* scores;
    uint32_t size;
} TriHeap;

static inline float vc_absf(float x) {
    return x < 0.0f ? -x : x;
}

static inline float vc_sqrtf(float x) {
#if defined(__clang__)
    return __builtin_sqrtf(x);
#else
    return (float)sqrt((double)x);
#endif
}

static inline float vc_powf(float a, float b) {
#if defined(__clang__)
    return __builtin_powf(a, b);
#else
    return (float)pow((double)a, (double)b);
#endif
}

static inline void vc_set_error(VertexCacheResult* r, const char* msg) {
    r->success = 0;
    if (!msg) return;
    for (int i = 0; i < 255 && msg[i]; i++) {
        r->error_message[i] = msg[i];
    }
}

static float vertex_score(int cache_pos, uint32_t active_tris, uint32_t cache_size) {
    if (active_tris == 0) {
        return -1.0f;
    }

    const float cache_decay_power = 1.5f;
    const float last_tri_score = 0.75f;
    const float valence_boost_scale = 2.0f;
    const float valence_boost_power = 0.5f;

    float score = 0.0f;

    if (cache_pos < 0) {
        score = 0.0f;
    } else {
        if ((uint32_t)cache_pos < 3) {
            score = last_tri_score;
        } else {
            float scaler = 1.0f / (float)(cache_size - 3);
            float v = 1.0f - ((float)(cache_pos - 3) * scaler);
            if (v < 0.0f) v = 0.0f;
            score = vc_powf(v, cache_decay_power);
        }
    }

    float valence = valence_boost_scale * vc_powf((float)active_tris, -valence_boost_power);
    score += valence;
    return score;
}

static void heap_swap(TriHeap* h, uint32_t a, uint32_t b) {
    uint32_t ta = h->heap[a];
    uint32_t tb = h->heap[b];
    h->heap[a] = tb;
    h->heap[b] = ta;
    h->pos[ta] = b;
    h->pos[tb] = a;
}

static void heap_sift_up(TriHeap* h, uint32_t idx) {
    while (idx > 0) {
        uint32_t parent = (idx - 1) >> 1;
        uint32_t t = h->heap[idx];
        uint32_t p = h->heap[parent];
        if (h->scores[t] <= h->scores[p]) {
            break;
        }
        heap_swap(h, idx, parent);
        idx = parent;
    }
}

static void heap_sift_down(TriHeap* h, uint32_t idx) {
    for (;;) {
        uint32_t left = idx * 2 + 1;
        if (left >= h->size) break;
        uint32_t right = left + 1;
        uint32_t best = left;

        if (right < h->size) {
            uint32_t tl = h->heap[left];
            uint32_t tr = h->heap[right];
            if (h->scores[tr] > h->scores[tl]) {
                best = right;
            }
        }

        uint32_t ti = h->heap[idx];
        uint32_t tb = h->heap[best];
        if (h->scores[ti] >= h->scores[tb]) {
            break;
        }

        heap_swap(h, idx, best);
        idx = best;
    }
}

static void heap_build(TriHeap* h, uint32_t tri_count) {
    h->size = tri_count;
    for (uint32_t i = 0; i < tri_count; i++) {
        h->heap[i] = i;
        h->pos[i] = i;
    }

    if (tri_count == 0) return;
    for (int32_t i = (int32_t)(tri_count / 2); i >= 0; i--) {
        heap_sift_down(h, (uint32_t)i);
    }
}

static int heap_pop_max(TriHeap* h, uint32_t* out_tri) {
    if (h->size == 0) return 0;
    uint32_t top = h->heap[0];
    h->size--;
    if (h->size > 0) {
        h->heap[0] = h->heap[h->size];
        h->pos[h->heap[0]] = 0;
        heap_sift_down(h, 0);
    }
    h->pos[top] = UINT32_MAX;
    *out_tri = top;
    return 1;
}

static void heap_update(TriHeap* h, uint32_t tri) {
    uint32_t p = h->pos[tri];
    if (p == UINT32_MAX) return;
    heap_sift_up(h, p);
    heap_sift_down(h, p);
}

WASM_EXPORT VertexCacheResult optimize_vertex_cache_forsyth(
    const uint32_t* indices,
    size_t index_count,
    size_t vertex_count,
    uint32_t cache_size
) {
    VertexCacheResult result = {0};

    if (!indices || index_count == 0 || vertex_count == 0) {
        vc_set_error(&result, "Invalid input parameters");
        return result;
    }

    if ((index_count % 3) != 0) {
        vc_set_error(&result, "Indices must be a triangle list");
        return result;
    }

    if (cache_size < 4) cache_size = 4;
    if (cache_size > 64) cache_size = 64;

    const uint32_t tri_count = (uint32_t)(index_count / 3);

    VCVertex* verts = (VCVertex*)wasm_malloc(vertex_count * sizeof(VCVertex));
    if (!verts) {
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }
    for (size_t v = 0; v < vertex_count; v++) {
        verts[v].score = 0.0f;
        verts[v].cache_pos = -1;
        verts[v].active_tris = 0;
        verts[v].tris = NULL;
        verts[v].tri_count = 0;
    }

    for (size_t i = 0; i < index_count; i++) {
        uint32_t vi = indices[i];
        if ((size_t)vi >= vertex_count) {
            wasm_free(verts);
            vc_set_error(&result, "Index out of range");
            return result;
        }
        verts[vi].active_tris++;
    }

    uint32_t* offsets = (uint32_t*)wasm_malloc((vertex_count + 1) * sizeof(uint32_t));
    if (!offsets) {
        wasm_free(verts);
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }

    uint32_t sum = 0;
    for (size_t v = 0; v < vertex_count; v++) {
        offsets[v] = sum;
        sum += verts[v].active_tris;
        verts[v].tri_count = verts[v].active_tris;
        verts[v].active_tris = verts[v].tri_count;
    }
    offsets[vertex_count] = sum;

    uint32_t* adjacency = (uint32_t*)wasm_malloc(sum * sizeof(uint32_t));
    if (!adjacency) {
        wasm_free(offsets);
        wasm_free(verts);
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }

    uint32_t* cursor = (uint32_t*)wasm_malloc(vertex_count * sizeof(uint32_t));
    if (!cursor) {
        wasm_free(adjacency);
        wasm_free(offsets);
        wasm_free(verts);
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }
    for (size_t v = 0; v < vertex_count; v++) {
        cursor[v] = offsets[v];
        verts[v].tris = &adjacency[offsets[v]];
    }

    for (uint32_t t = 0; t < tri_count; t++) {
        uint32_t a = indices[t * 3 + 0];
        uint32_t b = indices[t * 3 + 1];
        uint32_t c = indices[t * 3 + 2];
        adjacency[cursor[a]++] = t;
        adjacency[cursor[b]++] = t;
        adjacency[cursor[c]++] = t;
    }

    wasm_free(cursor);

    VCTriangle* tris = (VCTriangle*)wasm_malloc(tri_count * sizeof(VCTriangle));
    if (!tris) {
        wasm_free(adjacency);
        wasm_free(offsets);
        wasm_free(verts);
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }

    for (size_t v = 0; v < vertex_count; v++) {
        verts[v].score = vertex_score(-1, verts[v].active_tris, cache_size);
    }

    float* tri_scores = (float*)wasm_malloc(tri_count * sizeof(float));
    uint32_t* heap_arr = (uint32_t*)wasm_malloc(tri_count * sizeof(uint32_t));
    uint32_t* heap_pos = (uint32_t*)wasm_malloc(tri_count * sizeof(uint32_t));

    if (!tri_scores || !heap_arr || !heap_pos) {
        if (heap_pos) wasm_free(heap_pos);
        if (heap_arr) wasm_free(heap_arr);
        if (tri_scores) wasm_free(tri_scores);
        wasm_free(tris);
        wasm_free(adjacency);
        wasm_free(offsets);
        wasm_free(verts);
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }

    for (uint32_t t = 0; t < tri_count; t++) {
        tris[t].v[0] = indices[t * 3 + 0];
        tris[t].v[1] = indices[t * 3 + 1];
        tris[t].v[2] = indices[t * 3 + 2];
        tris[t].emitted = 0;

        float s = verts[tris[t].v[0]].score + verts[tris[t].v[1]].score + verts[tris[t].v[2]].score;
        tris[t].score = s;
        tri_scores[t] = s;
    }

    TriHeap heap;
    heap.heap = heap_arr;
    heap.pos = heap_pos;
    heap.scores = tri_scores;
    heap_build(&heap, tri_count);

    uint32_t* out = (uint32_t*)wasm_malloc(index_count * sizeof(uint32_t));
    if (!out) {
        wasm_free(heap_pos);
        wasm_free(heap_arr);
        wasm_free(tri_scores);
        wasm_free(tris);
        wasm_free(adjacency);
        wasm_free(offsets);
        wasm_free(verts);
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }

    int32_t* cache = (int32_t*)wasm_malloc(cache_size * sizeof(int32_t));
    if (!cache) {
        wasm_free(out);
        wasm_free(heap_pos);
        wasm_free(heap_arr);
        wasm_free(tri_scores);
        wasm_free(tris);
        wasm_free(adjacency);
        wasm_free(offsets);
        wasm_free(verts);
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }

    for (uint32_t i = 0; i < cache_size; i++) {
        cache[i] = -1;
    }

    uint32_t out_i = 0;

    uint8_t* touched = (uint8_t*)wasm_malloc(vertex_count);
    if (!touched) {
        wasm_free(cache);
        wasm_free(out);
        wasm_free(heap_pos);
        wasm_free(heap_arr);
        wasm_free(tri_scores);
        wasm_free(tris);
        wasm_free(adjacency);
        wasm_free(offsets);
        wasm_free(verts);
        vc_set_error(&result, "Memory allocation failed");
        return result;
    }

    for (size_t v = 0; v < vertex_count; v++) {
        touched[v] = 0;
    }

    for (uint32_t step = 0; step < tri_count; step++) {
        uint32_t tri;
        if (!heap_pop_max(&heap, &tri)) {
            break;
        }

        if (tris[tri].emitted) {
            step--;
            continue;
        }

        tris[tri].emitted = 1;

        out[out_i++] = tris[tri].v[0];
        out[out_i++] = tris[tri].v[1];
        out[out_i++] = tris[tri].v[2];

        for (int k = 0; k < 3; k++) {
            uint32_t v = tris[tri].v[k];
            if (verts[v].active_tris > 0) {
                verts[v].active_tris--;
            }
        }

        for (int k = 0; k < 3; k++) {
            int32_t v = (int32_t)tris[tri].v[k];
            int found = 0;
            for (uint32_t i = 0; i < cache_size; i++) {
                if (cache[i] == v) {
                    found = 1;
                    for (uint32_t j = i; j > 0; j--) {
                        cache[j] = cache[j - 1];
                    }
                    cache[0] = v;
                    break;
                }
            }
            if (!found) {
                for (uint32_t j = cache_size - 1; j > 0; j--) {
                    cache[j] = cache[j - 1];
                }
                cache[0] = v;
            }
        }

        for (uint32_t i = 0; i < cache_size; i++) {
            int32_t v = cache[i];
            if (v >= 0) {
                verts[(uint32_t)v].cache_pos = (int)i;
                touched[(uint32_t)v] = 1;
            }
        }

        for (int k = 0; k < 3; k++) {
            uint32_t v = tris[tri].v[k];
            touched[v] = 1;
        }

        for (uint32_t i = 0; i < cache_size; i++) {
            int32_t v = cache[i];
            if (v < 0) continue;
            uint32_t vi = (uint32_t)v;
            float ns = vertex_score(verts[vi].cache_pos, verts[vi].active_tris, cache_size);
            verts[vi].score = ns;
        }

        for (int k = 0; k < 3; k++) {
            uint32_t v = tris[tri].v[k];
            verts[v].score = vertex_score(verts[v].cache_pos, verts[v].active_tris, cache_size);
        }

        for (uint32_t i = 0; i < cache_size; i++) {
            int32_t v = cache[i];
            if (v < 0) continue;
            uint32_t vi = (uint32_t)v;
            if (!touched[vi]) continue;

            uint32_t count = verts[vi].tri_count;
            uint32_t* list = verts[vi].tris;
            for (uint32_t j = 0; j < count; j++) {
                uint32_t t = list[j];
                if (tris[t].emitted) continue;
                float s = verts[tris[t].v[0]].score + verts[tris[t].v[1]].score + verts[tris[t].v[2]].score;
                tris[t].score = s;
                tri_scores[t] = s;
                heap_update(&heap, t);
            }

            touched[vi] = 0;
        }

        for (int k = 0; k < 3; k++) {
            uint32_t v = tris[tri].v[k];
            if (!touched[v]) continue;
            uint32_t count = verts[v].tri_count;
            uint32_t* list = verts[v].tris;
            for (uint32_t j = 0; j < count; j++) {
                uint32_t t = list[j];
                if (tris[t].emitted) continue;
                float s = verts[tris[t].v[0]].score + verts[tris[t].v[1]].score + verts[tris[t].v[2]].score;
                tris[t].score = s;
                tri_scores[t] = s;
                heap_update(&heap, t);
            }
            touched[v] = 0;
        }
    }

    wasm_free(touched);
    wasm_free(cache);
    wasm_free(heap_pos);
    wasm_free(heap_arr);
    wasm_free(tri_scores);
    wasm_free(tris);
    wasm_free(adjacency);
    wasm_free(offsets);
    wasm_free(verts);

    result.indices = out;
    result.index_count = index_count;
    result.success = 1;
    return result;
}

WASM_EXPORT void free_vertex_cache_result(VertexCacheResult* result) {
    if (!result) return;
    if (result->indices) {
        wasm_free(result->indices);
    }
    result->indices = NULL;
    result->index_count = 0;
    result->success = 0;
}
