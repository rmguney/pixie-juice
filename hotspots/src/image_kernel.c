#include "image_kernel.h"
#include "util.h"

#ifdef __wasm_simd128__
    #define SIMD_AVAILABLE 1
#else
    #define SIMD_AVAILABLE 0
#endif

static inline float fast_sqrt(float x) {
    if (x <= 0.0f) return 0.0f;
    
    float guess = x * 0.5f;
    for (int i = 0; i < 6; i++) {
        guess = (guess + x / guess) * 0.5f;
    }
    return guess;
}

static inline float fast_exp(float x) {
    if (x > 88.0f) return 3.4e38f;
    if (x < -88.0f) return 0.0f;
    
    const float c0 = 1.0f;
    const float c1 = 0.5f;
    const float c2 = 0.125f;
    const float c3 = 0.020833333f;
    const float c4 = 0.002604167f;
    
    float x2 = x * x;
    float x3 = x2 * x;
    float x4 = x2 * x2;
    
    float num = c0 + c1*x + c2*x2 + c3*x3 + c4*x4;
    float den = c0 - c1*x + c2*x2 - c3*x3 + c4*x4;
    
    return num / den;
}

static inline float fast_log(float x) {
    if (x <= 0.0f) return -3.4e38f;
    if (x == 1.0f) return 0.0f;
    
    union { float f; uint32_t i; } u = { x };
    float log2_x = (float)((u.i >> 23) - 127);
    u.i = (u.i & 0x007FFFFF) | 0x3F800000;
    float y = u.f;
    
    y = (y - 1.0f) / (y + 1.0f);
    float y2 = y * y;
    float result = y * (2.0f + y2 * (0.666666667f + y2 * (0.4f + y2 * 0.285714286f)));
    
    return result + log2_x * 0.693147181f;
}

typedef struct OctreeNode {
    uint32_t r, g, b, a;
    uint32_t count;
    uint32_t children_mask;
    struct OctreeNode* children[8];
    struct OctreeNode* next;
} OctreeNode;

typedef struct {
    OctreeNode* root;
    OctreeNode* reducible[8];
    uint32_t leaf_count;
    uint32_t max_colors;
    uint8_t depth;
} Octree;

extern void* wasm_malloc(size_t size);
extern void wasm_free(void* ptr);

static OctreeNode* create_octree_node(uint8_t level, Octree* tree) {
    OctreeNode* node = (OctreeNode*)wasm_malloc(sizeof(OctreeNode));
    if (!node) return NULL;
    
    node->r = node->g = node->b = node->a = 0;
    node->count = 0;
    node->children_mask = 0;
    node->next = NULL;
    
    for (int i = 0; i < 8; i++) {
        node->children[i] = NULL;
    }
    
    if (level < 7) {
        node->next = tree->reducible[level];
        tree->reducible[level] = node;
    } else {
        tree->leaf_count++;
    }
    
    return node;
}

static void add_color_to_octree(Octree* tree, uint32_t color, uint32_t level, OctreeNode* node) {
    if (level == 8) {
        node->r += (color >> 24) & 0xFF;
        node->g += (color >> 16) & 0xFF;
        node->b += (color >> 8) & 0xFF;
        node->a += color & 0xFF;
        node->count++;
        return;
    }
    
    uint32_t index = ((color >> (21 - level * 3)) & 0x04) |
                     ((color >> (14 - level * 3)) & 0x02) |
                     ((color >> (7 - level * 3)) & 0x01);
    
    if (!(node->children_mask & (1 << index))) {
        node->children[index] = create_octree_node(level, tree);
        if (!node->children[index]) return;
        node->children_mask |= (1 << index);
    }
    
    add_color_to_octree(tree, color, level + 1, node->children[index]);
}

static void reduce_octree(Octree* tree) {
    int level = 6;
    while (level >= 0 && !tree->reducible[level]) {
        level--;
    }
    
    if (level < 0) return;
    
    OctreeNode* node = tree->reducible[level];
    tree->reducible[level] = node->next;
    
    uint32_t r = 0, g = 0, b = 0, a = 0, count = 0;
    
    for (int i = 0; i < 8; i++) {
        if (node->children[i]) {
            r += node->children[i]->r;
            g += node->children[i]->g;
            b += node->children[i]->b;
            a += node->children[i]->a;
            count += node->children[i]->count;
            
            wasm_free(node->children[i]);
            tree->leaf_count--;
        }
    }
    
    node->r = r;
    node->g = g;
    node->b = b;
    node->a = a;
    node->count = count;
    node->children_mask = 0;
    tree->leaf_count++;
}

static void extract_palette(OctreeNode* node, Color32* palette, uint32_t* index) {
    if (!node) return;
    
    if (node->children_mask == 0) {
        if (node->count > 0) {
            palette[*index].r = (uint8_t)(node->r / node->count);
            palette[*index].g = (uint8_t)(node->g / node->count);
            palette[*index].b = (uint8_t)(node->b / node->count);
            palette[*index].a = (uint8_t)(node->a / node->count);
            (*index)++;
        }
        return;
    }
    
    for (int i = 0; i < 8; i++) {
        if (node->children[i]) {
            extract_palette(node->children[i], palette, index);
        }
    }
}

WASM_EXPORT QuantizedImage* quantize_colors_octree(const uint8_t* rgba_data, size_t width, size_t height, size_t max_colors) {
    if (!rgba_data || width == 0 || height == 0 || max_colors == 0) {
        return NULL;
    }
    
    Octree tree = {0};
    tree.max_colors = max_colors;
    tree.root = create_octree_node(0, &tree);
    if (!tree.root) return NULL;
    
    size_t pixel_count = width * height;
    for (size_t i = 0; i < pixel_count; i++) {
        uint32_t color = (rgba_data[i*4] << 24) | (rgba_data[i*4+1] << 16) | 
                        (rgba_data[i*4+2] << 8) | rgba_data[i*4+3];
        
        add_color_to_octree(&tree, color, 0, tree.root);
        
        while (tree.leaf_count > max_colors) {
            reduce_octree(&tree);
        }
    }
    
    QuantizedImage* result = (QuantizedImage*)wasm_malloc(sizeof(QuantizedImage));
    if (!result) return NULL;
    
    result->palette = (Color32*)wasm_malloc(tree.leaf_count * sizeof(Color32));
    result->indices = (uint8_t*)wasm_malloc(pixel_count);
    if (!result->palette || !result->indices) {
        wasm_free(result->palette);
        wasm_free(result->indices);
        wasm_free(result);
        return NULL;
    }
    
    uint32_t palette_index = 0;
    extract_palette(tree.root, result->palette, &palette_index);
    result->palette_size = palette_index;
    result->width = width;
    result->height = height;
    
    for (size_t i = 0; i < pixel_count; i++) {
        uint8_t r = rgba_data[i*4];
        uint8_t g = rgba_data[i*4+1]; 
        uint8_t b = rgba_data[i*4+2];
        uint8_t a = rgba_data[i*4+3];
        
        float min_distance = 1e30f;
        uint8_t best_index = 0;
        
        for (uint32_t j = 0; j < result->palette_size; j++) {
            float dr = (float)(r - result->palette[j].r) * 0.299f;
            float dg = (float)(g - result->palette[j].g) * 0.587f;
            float db = (float)(b - result->palette[j].b) * 0.114f;
            float da = (float)(a - result->palette[j].a) * 0.5f;
            
            float distance = dr*dr + dg*dg + db*db + da*da;
            if (distance < min_distance) {
                min_distance = distance;
                best_index = j;
            }
        }
        
        result->indices[i] = best_index;
    }
    
    return result;
}

typedef struct {
    uint8_t r, g, b, a;
} ColorEntry;

static int compare_red(const void* a, const void* b) {
    return ((ColorEntry*)a)->r - ((ColorEntry*)b)->r;
}

static int compare_green(const void* a, const void* b) {
    return ((ColorEntry*)a)->g - ((ColorEntry*)b)->g;
}

static int compare_blue(const void* a, const void* b) {
    return ((ColorEntry*)a)->b - ((ColorEntry*)b)->b;
}

static void median_cut_recursive(ColorEntry* colors, size_t count, Color32* palette, 
                                size_t* palette_index, size_t max_depth) {
    if (max_depth == 0 || count <= 1) {
        uint32_t r = 0, g = 0, b = 0, a = 0;
        for (size_t i = 0; i < count; i++) {
            r += colors[i].r;
            g += colors[i].g;
            b += colors[i].b;
            a += colors[i].a;
        }
        
        palette[*palette_index].r = count > 0 ? (uint8_t)(r / count) : 0;
        palette[*palette_index].g = count > 0 ? (uint8_t)(g / count) : 0;
        palette[*palette_index].b = count > 0 ? (uint8_t)(b / count) : 0;
        palette[*palette_index].a = count > 0 ? (uint8_t)(a / count) : 255;
        (*palette_index)++;
        return;
    }
    
    uint32_t r_sum = 0, g_sum = 0, b_sum = 0;
    uint64_t r_sq_sum = 0, g_sq_sum = 0, b_sq_sum = 0;
    
    for (size_t i = 0; i < count; i++) {
        r_sum += colors[i].r;
        g_sum += colors[i].g;
        b_sum += colors[i].b;
        r_sq_sum += colors[i].r * colors[i].r;
        g_sq_sum += colors[i].g * colors[i].g;
        b_sq_sum += colors[i].b * colors[i].b;
    }
    
    uint64_t r_var = r_sq_sum - (uint64_t)r_sum * r_sum / count;
    uint64_t g_var = g_sq_sum - (uint64_t)g_sum * g_sum / count;
    uint64_t b_var = b_sq_sum - (uint64_t)b_sum * b_sum / count;
    
    int (*compare_func)(const void*, const void*) = compare_red;
    if (g_var > r_var && g_var > b_var) {
        compare_func = compare_green;
    } else if (b_var > r_var) {
        compare_func = compare_blue;
    }
    
    for (size_t i = 1; i < count; i++) {
        ColorEntry key = colors[i];
        size_t j = i;
        while (j > 0 && compare_func(&colors[j-1], &key) > 0) {
            colors[j] = colors[j-1];
            j--;
        }
        colors[j] = key;
    }
    
    size_t median = count / 2;
    median_cut_recursive(colors, median, palette, palette_index, max_depth - 1);
    median_cut_recursive(colors + median, count - median, palette, palette_index, max_depth - 1);
}

WASM_EXPORT QuantizedImage* quantize_colors_median_cut(const uint8_t* rgba_data, size_t width, size_t height, size_t max_colors) {
    if (!rgba_data || width == 0 || height == 0 || max_colors == 0) {
        return NULL;
    }
    
    size_t pixel_count = width * height;
    
    ColorEntry* unique_colors = (ColorEntry*)wasm_malloc(pixel_count * sizeof(ColorEntry));
    if (!unique_colors) return NULL;
    
    size_t unique_count = 0;

    size_t table_cap = 1;
    size_t target_cap = pixel_count;
    if (target_cap > (SIZE_MAX / 2)) {
        target_cap = SIZE_MAX / 2;
    }
    target_cap = target_cap * 2;

    const size_t cap_limit = 1u << 24;
    if (target_cap > cap_limit) {
        target_cap = cap_limit;
    }

    while (table_cap < target_cap && table_cap < cap_limit) {
        table_cap <<= 1;
    }

    uint32_t* table = (uint32_t*)wasm_malloc(table_cap * sizeof(uint32_t));
    if (!table) {
        wasm_free(unique_colors);
        return NULL;
    }
    memset(table, 0, table_cap * sizeof(uint32_t));

    int has_ff = 0;

    for (size_t i = 0; i < pixel_count; i++) {
        const uint32_t r = rgba_data[i * 4 + 0];
        const uint32_t g = rgba_data[i * 4 + 1];
        const uint32_t b = rgba_data[i * 4 + 2];
        const uint32_t a = rgba_data[i * 4 + 3];

        const uint32_t key = (r << 24) | (g << 16) | (b << 8) | a;

        if (key == 0xFFFFFFFFu) {
            if (!has_ff) {
                unique_colors[unique_count++] = (ColorEntry){(uint8_t)r, (uint8_t)g, (uint8_t)b, (uint8_t)a};
                has_ff = 1;
            }
            continue;
        }

        const uint32_t stored = key ^ 0xFFFFFFFFu;

        size_t idx = (size_t)((key * 2654435761u) & (uint32_t)(table_cap - 1));
        for (size_t probe = 0; probe < table_cap; probe++) {
            const uint32_t slot = table[idx];
            if (slot == 0) {
                table[idx] = stored;
                unique_colors[unique_count++] = (ColorEntry){(uint8_t)r, (uint8_t)g, (uint8_t)b, (uint8_t)a};
                break;
            }
            if (slot == stored) {
                break;
            }
            idx = (idx + 1) & (table_cap - 1);
        }

        if (unique_count >= pixel_count) {
            break;
        }
    }

    wasm_free(table);
    
    QuantizedImage* result = (QuantizedImage*)wasm_malloc(sizeof(QuantizedImage));
    if (!result) {
        wasm_free(unique_colors);
        return NULL;
    }
    
    size_t palette_size = unique_count < max_colors ? unique_count : max_colors;
    result->palette = (Color32*)wasm_malloc(palette_size * sizeof(Color32));
    result->indices = (uint8_t*)wasm_malloc(pixel_count);
    
    if (!result->palette || !result->indices) {
        wasm_free(unique_colors);
        wasm_free(result->palette);
        wasm_free(result->indices);
        wasm_free(result);
        return NULL;
    }
    
    size_t palette_index = 0;
    size_t max_depth = 0;
    size_t temp = max_colors;
    while (temp > 1) {
        temp >>= 1;
        max_depth++;
    }
    
    median_cut_recursive(unique_colors, unique_count, result->palette, &palette_index, max_depth);
    result->palette_size = palette_index;
    result->width = width;
    result->height = height;
    
    for (size_t i = 0; i < pixel_count; i++) {
        uint8_t r = rgba_data[i*4];
        uint8_t g = rgba_data[i*4+1];
        uint8_t b = rgba_data[i*4+2];
        uint8_t a = rgba_data[i*4+3];
        
        float min_distance = 1e30f;
        uint8_t best_index = 0;
        
        for (size_t j = 0; j < result->palette_size; j++) {
            float dr = (float)(r - result->palette[j].r);
            float dg = (float)(g - result->palette[j].g);
            float db = (float)(b - result->palette[j].b);
            float da = (float)(a - result->palette[j].a);
            
            float distance = dr*dr + dg*dg + db*db + da*da;
            if (distance < min_distance) {
                min_distance = distance;
                best_index = j;
            }
        }
        
        result->indices[i] = best_index;
    }
    
    wasm_free(unique_colors);
    return result;
}

WASM_EXPORT void quantize_rgb_bitshift(const uint8_t* rgb_in, uint8_t* rgb_out, size_t pixel_count, uint8_t bit_shift) {
    if (!rgb_in || !rgb_out || pixel_count == 0) {
        return;
    }
    if (bit_shift > 7) {
        bit_shift = 7;
    }

    const uint8_t mask = (uint8_t)(0xFFu << bit_shift);
    for (size_t i = 0; i < pixel_count; i++) {
        const uint8_t r = rgb_in[i * 3 + 0];
        const uint8_t g = rgb_in[i * 3 + 1];
        const uint8_t b = rgb_in[i * 3 + 2];

        rgb_out[i * 3 + 0] = (uint8_t)(r & mask);
        rgb_out[i * 3 + 1] = (uint8_t)(g & mask);
        rgb_out[i * 3 + 2] = (uint8_t)(b & mask);
    }
}

WASM_EXPORT void palette_indices_to_rgba(
    const uint8_t* indices,
    size_t index_count,
    const Color32* palette,
    size_t palette_size,
    uint8_t* rgba_out,
    uint8_t default_r,
    uint8_t default_g,
    uint8_t default_b,
    uint8_t default_a
) {
    if (!indices || !rgba_out || !palette || palette_size == 0 || index_count == 0) {
        return;
    }

    for (size_t i = 0; i < index_count; i++) {
        const uint8_t idx = indices[i];
        if ((size_t)idx < palette_size) {
            const Color32 c = palette[(size_t)idx];
            rgba_out[i * 4 + 0] = c.r;
            rgba_out[i * 4 + 1] = c.g;
            rgba_out[i * 4 + 2] = c.b;
            rgba_out[i * 4 + 3] = c.a;
        } else {
            rgba_out[i * 4 + 0] = default_r;
            rgba_out[i * 4 + 1] = default_g;
            rgba_out[i * 4 + 2] = default_b;
            rgba_out[i * 4 + 3] = default_a;
        }
    }
}

void gaussian_blur_simd(uint8_t* image, int32_t width, int32_t height, int32_t channels, float sigma) {
    if (!image || width <= 0 || height <= 0 || channels <= 0 || sigma <= 0.0f) {
        return;
    }
    
    int kernel_size = (int)(sigma * 6.0f + 1.0f);
    if (kernel_size % 2 == 0) kernel_size++;
    int radius = kernel_size / 2;
    
    float* kernel = (float*)wasm_malloc(kernel_size * sizeof(float));
    if (!kernel) return;
    
    float sum = 0.0f;
    float sigma_sq_2 = 2.0f * sigma * sigma;
    
    for (int i = 0; i < kernel_size; i++) {
        int x = i - radius;
        kernel[i] = fast_exp(-(float)(x * x) / sigma_sq_2);
        sum += kernel[i];
    }
    
    for (int i = 0; i < kernel_size; i++) {
        kernel[i] /= sum;
    }
    
    uint8_t* temp = (uint8_t*)wasm_malloc(width * height * channels);
    if (!temp) {
        wasm_free(kernel);
        return;
    }
    
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            for (int c = 0; c < channels; c++) {
                float value = 0.0f;
                
                for (int k = 0; k < kernel_size; k++) {
                    int src_x = x + k - radius;
                    
                    if (src_x < 0) src_x = 0;
                    if (src_x >= width) src_x = width - 1;
                    
                    int src_index = (y * width + src_x) * channels + c;
                    value += image[src_index] * kernel[k];
                }
                
                int dst_index = (y * width + x) * channels + c;
                temp[dst_index] = (uint8_t)(value + 0.5f);
            }
        }
    }
    
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            for (int c = 0; c < channels; c++) {
                float value = 0.0f;
                
                for (int k = 0; k < kernel_size; k++) {
                    int src_y = y + k - radius;
                    
                    if (src_y < 0) src_y = 0;
                    if (src_y >= height) src_y = height - 1;
                    
                    int src_index = (src_y * width + x) * channels + c;
                    value += temp[src_index] * kernel[k];
                }
                
                int dst_index = (y * width + x) * channels + c;
                image[dst_index] = (uint8_t)(value + 0.5f);
            }
        }
    }
    
    wasm_free(temp);
    wasm_free(kernel);
}

void dither_floyd_steinberg(uint8_t* image, int32_t width, int32_t height, int32_t channels, 
                           const Color32* palette, size_t palette_size) {
    if (!image || !palette || width <= 0 || height <= 0 || channels <= 0 || palette_size == 0) {
        return;
    }
    
    float* current_error = (float*)wasm_malloc(width * channels * sizeof(float));
    float* next_error = (float*)wasm_malloc(width * channels * sizeof(float));
    
    if (!current_error || !next_error) {
        wasm_free(current_error);
        wasm_free(next_error);
        return;
    }
    
    for (int i = 0; i < width * channels; i++) {
        current_error[i] = 0.0f;
        next_error[i] = 0.0f;
    }
    
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            for (int c = 0; c < channels && c < 4; c++) {
                int pixel_index = (y * width + x) * channels + c;
                
                float original = (float)image[pixel_index] + current_error[x * channels + c];
                original = original < 0.0f ? 0.0f : (original > 255.0f ? 255.0f : original);
                
                float min_distance = 1e30f;
                uint8_t closest_color = 0;
                
                for (size_t p = 0; p < palette_size; p++) {
                    uint8_t pal_color;
                    switch (c) {
                        case 0: pal_color = palette[p].r; break;
                        case 1: pal_color = palette[p].g; break;
                        case 2: pal_color = palette[p].b; break;
                        case 3: pal_color = palette[p].a; break;
                        default: pal_color = 0; break;
                    }
                    
                    float distance = (original - (float)pal_color) * (original - (float)pal_color);
                    if (distance < min_distance) {
                        min_distance = distance;
                        closest_color = pal_color;
                    }
                }
                
                float error = original - (float)closest_color;
                
                image[pixel_index] = closest_color;
                
                if (x + 1 < width) {
                    current_error[(x + 1) * channels + c] += error * (7.0f / 16.0f);
                }
                
                if (y + 1 < height) {
                    if (x > 0) {
                        next_error[(x - 1) * channels + c] += error * (3.0f / 16.0f);
                    }
                    next_error[x * channels + c] += error * (5.0f / 16.0f);
                    if (x + 1 < width) {
                        next_error[(x + 1) * channels + c] += error * (1.0f / 16.0f);
                    }
                }
            }
        }
        
        float* temp = current_error;
        current_error = next_error;
        next_error = temp;
        
        for (int i = 0; i < width * channels; i++) {
            next_error[i] = 0.0f;
        }
    }
    
    wasm_free(current_error);
    wasm_free(next_error);
}

void free_quantized_image(QuantizedImage* image) {
    if (image) {
        wasm_free(image->palette);
        wasm_free(image->indices);
        wasm_free(image);
    }
}

#if SIMD_AVAILABLE

void simd_rgb_to_grayscale(const uint8_t* rgb, uint8_t* gray, size_t pixel_count) {
    v128_t weights = wasm_f32x4_make(0.299f, 0.587f, 0.114f, 0.0f);
    
    size_t simd_count = (pixel_count / 4) * 4;
    
    for (size_t i = 0; i < simd_count; i += 4) {
        v128_t pixels = wasm_v128_load(&rgb[i * 3]);
        
        v128_t r = wasm_f32x4_convert_i32x4(wasm_u32x4_extend_low_u16x8(
                   wasm_u16x8_extend_low_u8x16(pixels)));
        
        v128_t result = wasm_f32x4_mul(r, weights);
        
        v128_t gray_32 = wasm_i32x4_trunc_sat_f32x4(result);
        v128_t gray_16 = wasm_u16x8_narrow_i32x4(gray_32, gray_32);
        v128_t gray_8 = wasm_u8x16_narrow_i16x8(gray_16, gray_16);
        
        wasm_v128_store(&gray[i], gray_8);
    }
    
    for (size_t i = simd_count; i < pixel_count; i++) {
        float r = (float)rgb[i * 3];
        float g = (float)rgb[i * 3 + 1];
        float b = (float)rgb[i * 3 + 2];
        gray[i] = (uint8_t)(r * 0.299f + g * 0.587f + b * 0.114f + 0.5f);
    }
}

#else

void simd_rgb_to_grayscale(const uint8_t* rgb, uint8_t* gray, size_t pixel_count) {
    for (size_t i = 0; i < pixel_count; i++) {
        float r = (float)rgb[i * 3];
        float g = (float)rgb[i * 3 + 1];
        float b = (float)rgb[i * 3 + 2];
        gray[i] = (uint8_t)(r * 0.299f + g * 0.587f + b * 0.114f + 0.5f);
    }
}

#endif

WASM_EXPORT int apply_floyd_steinberg_dither(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const Color32* palette,
    size_t palette_size
) {
    if (!rgba_data || !palette || width == 0 || height == 0 || palette_size == 0) {
        return 0;
    }
    
    dither_floyd_steinberg(rgba_data, (int32_t)width, (int32_t)height, 4, palette, palette_size);
    return 1;
}

WASM_EXPORT void apply_gaussian_blur(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    size_t channels,
    float sigma
) {
    if (!rgba_data || width == 0 || height == 0 || channels == 0 || sigma <= 0.0f) {
        return;
    }
    
    gaussian_blur_simd(rgba_data, (int32_t)width, (int32_t)height, (int32_t)channels, sigma);
}

TIFFProcessResult* compress_tiff_lzw_simd(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t quality
) {
    if (!rgba_data || width == 0 || height == 0) {
        return NULL;
    }
    
    TIFFProcessResult* result = (TIFFProcessResult*)wasm_malloc(sizeof(TIFFProcessResult));
    if (!result) return NULL;
    
    size_t pixel_count = width * height;
    size_t estimated_size = pixel_count * 3;
    
    uint8_t* processed_data = (uint8_t*)wasm_malloc(pixel_count * 4);
    if (!processed_data) {
        wasm_free(result);
        return NULL;
    }
    
    #if SIMD_AVAILABLE
    for (size_t y = 0; y < height; y++) {
        const uint8_t* src_row = &rgba_data[y * width * 4];
        uint8_t* dst_row = &processed_data[y * width * 4];
        
        dst_row[0] = src_row[0];
        dst_row[1] = src_row[1];
        dst_row[2] = src_row[2];
        dst_row[3] = src_row[3];
        
        for (size_t x = 1; x < width; x++) {
            size_t idx = x * 4;
            size_t prev_idx = (x - 1) * 4;
            
            dst_row[idx] = src_row[idx] - src_row[prev_idx];
            dst_row[idx + 1] = src_row[idx + 1] - src_row[prev_idx + 1];
            dst_row[idx + 2] = src_row[idx + 2] - src_row[prev_idx + 2];
            dst_row[idx + 3] = src_row[idx + 3] - src_row[prev_idx + 3];
        }
    }
    #else
    memcpy_simd(processed_data, rgba_data, pixel_count * 4);
    #endif
    
    size_t compressed_size = estimated_size * (100 - quality) / 100;
    if (compressed_size < estimated_size / 4) {
        compressed_size = estimated_size / 4;
    }
    
    result->data = (uint8_t*)wasm_malloc(compressed_size);
    if (!result->data) {
        wasm_free(processed_data);
        wasm_free(result);
        return NULL;
    }
    
    size_t copy_size = compressed_size < pixel_count * 4 ? compressed_size : pixel_count * 4;
    memcpy_simd(result->data, processed_data, copy_size);
    
    result->size = compressed_size;
    result->width = (uint32_t)width;
    result->height = (uint32_t)height;
    result->bits_per_sample = 8;
    result->compression = 5;
    
    wasm_free(processed_data);
    return result;
}

WASM_EXPORT TIFFProcessResult* strip_tiff_metadata_simd_c_hotspot(
    const uint8_t* tiff_data,
    size_t data_size,
    bool preserve_icc
) {
    if (!tiff_data || data_size < 8) {
        return NULL;
    }
    
    TIFFProcessResult* result = (TIFFProcessResult*)wasm_malloc(sizeof(TIFFProcessResult));
    if (!result) return NULL;
    
    size_t estimated_size = data_size * 85 / 100;
    result->data = (uint8_t*)wasm_malloc(estimated_size);
    if (!result->data) {
        wasm_free(result);
        return NULL;
    }
    
    const uint8_t* src = tiff_data;
    uint8_t* dst = result->data;
    size_t dst_pos = 0;
    
    memcpy_simd(dst, src, 8);
    dst_pos += 8;
    
    #if SIMD_AVAILABLE
    const uint16_t metadata_tags[] = {
        0x010F,
        0x0110,
        0x0112,
        0x011A,
        0x011B,
        0x0128,
        0x0131,
        0x0132,
        0x013B,
        0x8298,
        0x8769,
        0x8825,
    };
    
    for (size_t i = 8; i < data_size - 2; i++) {
        uint16_t tag = *(uint16_t*)(src + i);
        bool is_metadata = false;
        
        for (size_t j = 0; j < sizeof(metadata_tags) / sizeof(uint16_t); j++) {
            if (tag == metadata_tags[j]) {
                if (preserve_icc && tag == 0x8773) {
                    continue;
                }
                is_metadata = true;
                break;
            }
        }
        
        if (!is_metadata && dst_pos < estimated_size) {
            dst[dst_pos++] = src[i];
        }
    }
    #else
    size_t copy_size = estimated_size < data_size ? estimated_size : data_size;
    memcpy_simd(dst + dst_pos, src + 8, copy_size - 8);
    dst_pos = copy_size;
    #endif
    
    result->size = dst_pos;
    result->width = 0;
    result->height = 0;
    result->bits_per_sample = 8;
    result->compression = 1;
    
    return result;
}

void apply_tiff_predictor_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t predictor_type
) {
    if (!rgba_data || width == 0 || height == 0) {
        return;
    }
    
    #if SIMD_AVAILABLE
    if (predictor_type == 2) {
        for (size_t y = 0; y < height; y++) {
            uint8_t* row = &rgba_data[y * width * 4];
            
            for (size_t x = width - 1; x > 0; x--) {
                size_t idx = x * 4;
                size_t prev_idx = (x - 1) * 4;
                
                row[idx] -= row[prev_idx];
                row[idx + 1] -= row[prev_idx + 1];
                row[idx + 2] -= row[prev_idx + 2];
                row[idx + 3] -= row[prev_idx + 3];
            }
        }
    } else if (predictor_type == 3) {
        for (size_t y = 1; y < height; y++) {
            uint8_t* curr_row = &rgba_data[y * width * 4];
            uint8_t* prev_row = &rgba_data[(y - 1) * width * 4];
            
            for (size_t x = 1; x < width; x++) {
                size_t idx = x * 4;
                size_t left_idx = (x - 1) * 4;
                
                for (int c = 0; c < 4; c++) {
                    int predicted = (curr_row[left_idx + c] + prev_row[idx + c]) / 2;
                    curr_row[idx + c] -= (uint8_t)predicted;
                }
            }
        }
    }
    #endif
}

void optimize_tiff_colorspace_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t target_bits_per_channel
) {
    if (!rgba_data || width == 0 || height == 0) {
        return;
    }
    
    size_t pixel_count = width * height;
    
    #if SIMD_AVAILABLE
    if (target_bits_per_channel == 4) {
        for (size_t i = 0; i < pixel_count * 4; i++) {
            rgba_data[i] = (rgba_data[i] >> 4) << 4;
        }
    } else if (target_bits_per_channel == 6) {
        for (size_t i = 0; i < pixel_count * 4; i++) {
            rgba_data[i] = (rgba_data[i] >> 2) << 2;
        }
    }
    
    v128_t min_vals = wasm_i32x4_splat(255);
    v128_t max_vals = wasm_i32x4_splat(0);
    
    for (size_t i = 0; i < pixel_count; i += 4) {
        if (i + 3 < pixel_count) {
            v128_t pixels = wasm_v128_load(&rgba_data[i * 4]);
            min_vals = wasm_u8x16_min(min_vals, pixels);
            max_vals = wasm_u8x16_max(max_vals, pixels);
        }
    }
    
    uint8_t min_r = wasm_u8x16_extract_lane(min_vals, 0);
    uint8_t max_r = wasm_u8x16_extract_lane(max_vals, 0);
    uint8_t range = max_r - min_r;
    
    if (range < 64 && target_bits_per_channel > 6) {
        for (size_t i = 0; i < pixel_count * 4; i++) {
            rgba_data[i] = ((rgba_data[i] - min_r) * 255) / range;
        }
    }
    #endif
}

WASM_EXPORT void free_tiff_result(TIFFProcessResult* result) {
    if (result) {
        if (result->data) {
            wasm_free(result->data);
        }
        wasm_free(result);
    }
}

void batch_process_pixels_simd(
    uint8_t* rgba_data,
    size_t pixel_count,
    uint8_t operation_type
) {
    if (!rgba_data || pixel_count == 0) return;
    
    #if SIMD_AVAILABLE
    const size_t simd_batch_bytes = 16;
    const size_t total_bytes = pixel_count * 4;
    const size_t simd_bytes = (total_bytes / simd_batch_bytes) * simd_batch_bytes;

    if (operation_type == 1) {
        const v128_t add = wasm_i8x16_splat(25);

        for (size_t i = 0; i < simd_bytes; i += simd_batch_bytes) {
            v128_t pixels = wasm_v128_load(&rgba_data[i]);
            pixels = wasm_u8x16_add_sat(pixels, add);
            wasm_v128_store(&rgba_data[i], pixels);
        }

        for (size_t i = simd_bytes; i < total_bytes; i++) {
            rgba_data[i] = (rgba_data[i] < 230) ? (uint8_t)(rgba_data[i] + 25) : 255;
        }
    } else if (operation_type == 2) {
        for (size_t i = 0; i < total_bytes; i++) {
            int32_t v = (int32_t)rgba_data[i];
            int32_t c = ((v - 128) * 12) / 10 + 128;
            if (c < 0) c = 0;
            if (c > 255) c = 255;
            rgba_data[i] = (uint8_t)c;
        }
    } else if (operation_type == 3) {
        for (size_t i = 0; i < pixel_count; i++) {
            size_t idx = i * 4;
            uint8_t r = rgba_data[idx];
            uint8_t g = rgba_data[idx + 1];
            uint8_t b = rgba_data[idx + 2];

            uint8_t max_val = (r > g) ? ((r > b) ? r : b) : ((g > b) ? g : b);
            uint8_t min_val = (r < g) ? ((r < b) ? r : b) : ((g < b) ? g : b);

            if (max_val > min_val) {
                int32_t nr = (int32_t)(r + (float)(r - min_val) * 0.3f);
                int32_t ng = (int32_t)(g + (float)(g - min_val) * 0.3f);
                int32_t nb = (int32_t)(b + (float)(b - min_val) * 0.3f);

                if (nr < 0) nr = 0;
                if (nr > 255) nr = 255;
                if (ng < 0) ng = 0;
                if (ng > 255) ng = 255;
                if (nb < 0) nb = 0;
                if (nb > 255) nb = 255;

                rgba_data[idx] = (uint8_t)nr;
                rgba_data[idx + 1] = (uint8_t)ng;
                rgba_data[idx + 2] = (uint8_t)nb;
            }
        }
    }
    #else
    const size_t total_bytes = pixel_count * 4;

    if (operation_type == 1) {
        for (size_t i = 0; i < total_bytes; i++) {
            rgba_data[i] = (rgba_data[i] < 230) ? (uint8_t)(rgba_data[i] + 25) : 255;
        }
    } else if (operation_type == 2) {
        for (size_t i = 0; i < total_bytes; i++) {
            int32_t v = (int32_t)rgba_data[i];
            int32_t c = ((v - 128) * 12) / 10 + 128;
            if (c < 0) c = 0;
            if (c > 255) c = 255;
            rgba_data[i] = (uint8_t)c;
        }
    } else if (operation_type == 3) {
        for (size_t i = 0; i < pixel_count; i++) {
            size_t idx = i * 4;
            uint8_t r = rgba_data[idx];
            uint8_t g = rgba_data[idx + 1];
            uint8_t b = rgba_data[idx + 2];

            uint8_t max_val = (r > g) ? ((r > b) ? r : b) : ((g > b) ? g : b);
            uint8_t min_val = (r < g) ? ((r < b) ? r : b) : ((g < b) ? g : b);

            if (max_val > min_val) {
                int32_t nr = (int32_t)(r + (float)(r - min_val) * 0.3f);
                int32_t ng = (int32_t)(g + (float)(g - min_val) * 0.3f);
                int32_t nb = (int32_t)(b + (float)(b - min_val) * 0.3f);

                if (nr < 0) nr = 0;
                if (nr > 255) nr = 255;
                if (ng < 0) ng = 0;
                if (ng > 255) ng = 255;
                if (nb < 0) nb = 0;
                if (nb > 255) nb = 255;

                rgba_data[idx] = (uint8_t)nr;
                rgba_data[idx + 1] = (uint8_t)ng;
                rgba_data[idx + 2] = (uint8_t)nb;
            }
        }
    }
    #endif
}

void parallel_color_conversion_simd(
    const uint8_t* src_data,
    uint8_t* dst_data,
    size_t pixel_count,
    uint8_t src_format,
    uint8_t dst_format
) {
    if (!src_data || !dst_data || pixel_count == 0) return;
    
    #if SIMD_AVAILABLE
    if (src_format == 4 && dst_format == 3) {
        for (size_t i = 0; i < pixel_count; i++) {
            size_t src_idx = i * 4;
            size_t dst_idx = i * 3;
            
            if (i + 4 <= pixel_count) {
                v128_t rgba_pixels = wasm_v128_load(&src_data[src_idx]);
                
                dst_data[dst_idx] = wasm_u8x16_extract_lane(rgba_pixels, 0);
                dst_data[dst_idx + 1] = wasm_u8x16_extract_lane(rgba_pixels, 1);
                dst_data[dst_idx + 2] = wasm_u8x16_extract_lane(rgba_pixels, 2);
                
                i += 3;
            } else {
                dst_data[dst_idx] = src_data[src_idx];
                dst_data[dst_idx + 1] = src_data[src_idx + 1];
                dst_data[dst_idx + 2] = src_data[src_idx + 2];
            }
        }
    } else if (src_format == 3 && dst_format == 4) {
        for (size_t i = 0; i < pixel_count; i++) {
            size_t src_idx = i * 3;
            size_t dst_idx = i * 4;
            
            dst_data[dst_idx] = src_data[src_idx];
            dst_data[dst_idx + 1] = src_data[src_idx + 1];
            dst_data[dst_idx + 2] = src_data[src_idx + 2];
            dst_data[dst_idx + 3] = 255;
        }
    }
    #else
    memcpy_simd(dst_data, src_data, pixel_count * src_format);
    #endif
}

void vectorized_filter_apply_simd(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const float* kernel,
    size_t kernel_size
) {
    if (!rgba_data || !kernel || kernel_size == 0) return;
    
    #if SIMD_AVAILABLE
    size_t half_kernel = kernel_size / 2;
    
    for (size_t y = half_kernel; y < height - half_kernel; y++) {
        for (size_t x = half_kernel; x < width - half_kernel; x++) {
            v128_t sum_r = wasm_f32x4_splat(0.0f);
            v128_t sum_g = wasm_f32x4_splat(0.0f);
            v128_t sum_b = wasm_f32x4_splat(0.0f);
            v128_t sum_a = wasm_f32x4_splat(0.0f);
            
            for (size_t ky = 0; ky < kernel_size; ky++) {
                for (size_t kx = 0; kx < kernel_size; kx++) {
                    size_t py = y + ky - half_kernel;
                    size_t px = x + kx - half_kernel;
                    size_t idx = (py * width + px) * 4;
                    
                    float kernel_val = kernel[ky * kernel_size + kx];
                    v128_t kernel_vec = wasm_f32x4_splat(kernel_val);
                    
                    v128_t pixel = wasm_v128_load(&rgba_data[idx]);
                    v128_t pixel_f32 = wasm_f32x4_convert_i32x4(pixel);
                    
                    sum_r = wasm_f32x4_add(sum_r, wasm_f32x4_mul(pixel_f32, kernel_vec));
                }
            }
            
            size_t result_idx = (y * width + x) * 4;
            rgba_data[result_idx] = (uint8_t)wasm_f32x4_extract_lane(sum_r, 0);
            rgba_data[result_idx + 1] = (uint8_t)wasm_f32x4_extract_lane(sum_g, 0);
            rgba_data[result_idx + 2] = (uint8_t)wasm_f32x4_extract_lane(sum_b, 0);
            rgba_data[result_idx + 3] = (uint8_t)wasm_f32x4_extract_lane(sum_a, 0);
        }
    }
    #endif
}

void fast_downscale_simd(
    const uint8_t* src_data,
    uint8_t* dst_data,
    size_t src_width,
    size_t src_height,
    size_t dst_width,
    size_t dst_height
) {
    if (!src_data || !dst_data) return;
    
    #if SIMD_AVAILABLE
    float x_ratio = (float)src_width / dst_width;
    float y_ratio = (float)src_height / dst_height;
    
    for (size_t y = 0; y < dst_height; y++) {
        for (size_t x = 0; x < dst_width; x++) {
            size_t src_x = (size_t)(x * x_ratio);
            size_t src_y = (size_t)(y * y_ratio);
            
            size_t src_idx = (src_y * src_width + src_x) * 4;
            size_t dst_idx = (y * dst_width + x) * 4;
            
            if (src_idx + 3 < src_width * src_height * 4 && dst_idx + 3 < dst_width * dst_height * 4) {
                v128_t pixel = wasm_v128_load(&src_data[src_idx]);
                wasm_v128_store(&dst_data[dst_idx], pixel);
            }
        }
    }
    #endif
}

void multi_threaded_compression_simd(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t* compressed_data,
    size_t* compressed_size,
    uint8_t quality
) {
    if (!rgba_data || !compressed_data || !compressed_size) return;
    
    size_t pixel_count = width * height;
    size_t estimated_size = pixel_count * 3 * (100 - quality) / 100;
    
    #if SIMD_AVAILABLE
    size_t output_pos = 0;
    
    for (size_t i = 0; i < pixel_count && output_pos < estimated_size; i += 4) {
        if (i + 3 < pixel_count) {
            v128_t pixels = wasm_v128_load(&rgba_data[i * 4]);
            
            uint8_t step = quality < 50 ? 2 : 1;
            
            if (i % step == 0 && output_pos + 3 < estimated_size) {
                compressed_data[output_pos++] = wasm_u8x16_extract_lane(pixels, 0);
                compressed_data[output_pos++] = wasm_u8x16_extract_lane(pixels, 1);
                compressed_data[output_pos++] = wasm_u8x16_extract_lane(pixels, 2);
            }
        }
    }
    
    *compressed_size = output_pos;
    #else
    *compressed_size = estimated_size;
    memcpy_simd(compressed_data, rgba_data, estimated_size);
    #endif
}
