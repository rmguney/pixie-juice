#include "image_kernel.h"
#include "util.h"

// Only include standard headers for native builds
#ifndef __wasm__
    #include <stdlib.h>
    #include <string.h>
    #include <math.h>
#endif

#ifdef _MSC_VER
    #ifndef __wasm__
        #include <intrin.h>
    #endif
#else
    #ifndef __wasm__
        #include <x86intrin.h>
    #endif
#endif

// WASM-compatible implementations
#ifdef __wasm__
    // Math functions for WASM
    static inline double sqrt(double x) {
        if (x < 0.0) return 0.0;
        if (x == 0.0) return 0.0;
        double guess = x * 0.5;
        for (int i = 0; i < 10; i++) {
            guess = (guess + x / guess) * 0.5;
        }
        return guess;
    }
    
    static inline double pow(double base, double exp) {
        if (exp == 0.0) return 1.0;
        if (exp == 1.0) return base;
        if (exp == 2.0) return base * base;
        // Simplified power function for common cases
        double result = 1.0;
        for (int i = 0; i < (int)exp && i < 10; i++) {
            result *= base;
        }
        return result;
    }
    
    static inline float expf(float x) {
        // Simple exponential approximation: e^x ≈ 1 + x + x²/2! + x³/3! + ...
        if (x > 10.0f) return 22026.5f; // e^10 ≈ 22026
        if (x < -10.0f) return 0.0f;
        
        float result = 1.0f;
        float term = 1.0f;
        for (int i = 1; i <= 8; i++) {
            term *= x / i;
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
    
    static inline float powf(float base, float exp) {
        if (exp == 0.0f) return 1.0f;
        if (exp == 1.0f) return base;
        if (exp == 2.0f) return base * base;
        // Simplified power function for common cases
        float result = 1.0f;
        for (int i = 0; i < (int)exp && i < 10; i++) {
            result *= base;
        }
        return result;
    }
    
    // Memory operations for WASM
    static void* memcpy_wasm(void* dest, const void* src, size_t n) {
        char* d = (char*)dest;
        const char* s = (const char*)src;
        for (size_t i = 0; i < n; i++) {
            d[i] = s[i];
        }
        return dest;
    }
    
    static void* memset_wasm(void* s, int c, size_t n) {
        char* p = (char*)s;
        for (size_t i = 0; i < n; i++) {
            p[i] = (char)c;
        }
        return s;
    }
    
    // Memory allocation for WASM (simple static buffers)
    static char wasm_buffer[1024 * 1024]; // 1MB buffer
    static size_t wasm_buffer_offset = 0;
    
    static void* malloc_wasm(size_t size) {
        if (wasm_buffer_offset + size > sizeof(wasm_buffer)) {
            return 0; // Out of memory
        }
        void* ptr = &wasm_buffer[wasm_buffer_offset];
        wasm_buffer_offset += (size + 7) & ~7; // 8-byte align
        return ptr;
    }
    
    static void* calloc_wasm(size_t count, size_t size) {
        size_t total = count * size;
        void* ptr = malloc_wasm(total);
        if (ptr) {
            memset_wasm(ptr, 0, total);
        }
        return ptr;
    }
    
    static void free_wasm(void* ptr) {
        // Simple implementation - no actual freeing
        (void)ptr;
    }
    
    // Sorting function for WASM
    static void qsort_wasm(void* base, size_t num, size_t size, int (*cmp)(const void*, const void*)) {
        // Simple bubble sort for WASM
        char* array = (char*)base;
        for (size_t i = 0; i < num - 1; i++) {
            for (size_t j = 0; j < num - i - 1; j++) {
                if (cmp(array + j * size, array + (j + 1) * size) > 0) {
                    // Swap elements
                    for (size_t k = 0; k < size; k++) {
                        char temp = array[j * size + k];
                        array[j * size + k] = array[(j + 1) * size + k];
                        array[(j + 1) * size + k] = temp;
                    }
                }
            }
        }
    }
    
    // WASM SIMD support (if available)
    #ifdef __wasm_simd128__
        #include <wasm_simd128.h>
        
        // WASM SIMD wrapper types to match x86 interface
        typedef v128_t __m128i;
        typedef v128_t __m128;
        
        // Integer SIMD intrinsics
        static inline __m128i _mm_set_epi32(int e3, int e2, int e1, int e0) {
            return wasm_i32x4_make(e0, e1, e2, e3);
        }
        
        static inline __m128i _mm_sub_epi32(__m128i a, __m128i b) {
            return wasm_i32x4_sub(a, b);
        }
        
        static inline __m128i _mm_mullo_epi32(__m128i a, __m128i b) {
            return wasm_i32x4_mul(a, b);
        }
        
        static inline __m128i _mm_add_epi32(__m128i a, __m128i b) {
            return wasm_i32x4_add(a, b);
        }
        
        static inline int _mm_extract_epi32(__m128i a, int imm) {
            // WASM requires compile-time constants
            switch (imm & 3) {
                case 0: return wasm_i32x4_extract_lane(a, 0);
                case 1: return wasm_i32x4_extract_lane(a, 1);
                case 2: return wasm_i32x4_extract_lane(a, 2);
                case 3: return wasm_i32x4_extract_lane(a, 3);
                default: return wasm_i32x4_extract_lane(a, 0);
            }
        }
        
        // Float SIMD intrinsics
        static inline __m128 _mm_set_ps(float e3, float e2, float e1, float e0) {
            return wasm_f32x4_make(e0, e1, e2, e3);
        }
        
        static inline __m128 _mm_set1_ps(float a) {
            return wasm_f32x4_splat(a);
        }
        
        static inline __m128 _mm_setzero_ps(void) {
            return wasm_f32x4_splat(0.0f);
        }
        
        static inline __m128 _mm_add_ps(__m128 a, __m128 b) {
            return wasm_f32x4_add(a, b);
        }
        
        static inline __m128 _mm_sub_ps(__m128 a, __m128 b) {
            return wasm_f32x4_sub(a, b);
        }
        
        static inline __m128 _mm_mul_ps(__m128 a, __m128 b) {
            return wasm_f32x4_mul(a, b);
        }
        
        static inline __m128 _mm_div_ps(__m128 a, __m128 b) {
            return wasm_f32x4_div(a, b);
        }
        
        static inline void _mm_store_ps(float* mem_addr, __m128 a) {
            wasm_v128_store(mem_addr, a);
        }
        
        static inline __m128 _mm_load_ps(const float* mem_addr) {
            return wasm_v128_load(mem_addr);
        }
        
        static inline float _mm_cvtss_f32(__m128 a) {
            return wasm_f32x4_extract_lane(a, 0);
        }
        
        // Horizontal add emulation for WASM
        static inline __m128 _mm_hadd_ps(__m128 a, __m128 b) {
            // Simple horizontal add approximation
            float a0 = wasm_f32x4_extract_lane(a, 0) + wasm_f32x4_extract_lane(a, 1);
            float a1 = wasm_f32x4_extract_lane(a, 2) + wasm_f32x4_extract_lane(a, 3);
            float b0 = wasm_f32x4_extract_lane(b, 0) + wasm_f32x4_extract_lane(b, 1);
            float b1 = wasm_f32x4_extract_lane(b, 2) + wasm_f32x4_extract_lane(b, 3);
            return wasm_f32x4_make(a0, a1, b0, b1);
        }
        
        // Dot product emulation for WASM (duplicate from math_kernel.c)
        static inline __m128 _mm_dp_ps(__m128 a, __m128 b, unsigned int mask) {
            // For simplicity, just do a basic dot product and splat
            __m128 mul = wasm_f32x4_mul(a, b);
            float result = wasm_f32x4_extract_lane(mul, 0) + 
                          wasm_f32x4_extract_lane(mul, 1) + 
                          wasm_f32x4_extract_lane(mul, 2) + 
                          wasm_f32x4_extract_lane(mul, 3);
            return wasm_f32x4_splat(result);
        }
    #endif
    
    // Constants for WASM
    #define INT_MAX 2147483647
    
    #define memcpy memcpy_wasm
    #define memset memset_wasm
    #define malloc malloc_wasm
    #define calloc calloc_wasm
    #define free free_wasm
    #define qsort qsort_wasm
#endif

// Octree node for color quantization
typedef struct OctreeNode {
    struct OctreeNode* children[8];
    uint32_t red_sum;
    uint32_t green_sum;
    uint32_t blue_sum;
    uint32_t pixel_count;
    int is_leaf;
    int level;
} OctreeNode;

// Feature detection
static int has_sse = -1;

static void detect_simd_features() {
    if (has_sse == -1) {
#ifdef __wasm__
        // WASM doesn't support x86 CPU detection
        has_sse = 0;
#else
        int cpuinfo[4];
#ifdef _MSC_VER
        __cpuid(cpuinfo, 1);
#else
        __builtin_cpu_init();
        cpuinfo[3] = __builtin_cpu_supports("sse") ? (1 << 25) : 0;
#endif
        has_sse = (cpuinfo[3] & (1 << 25)) != 0;
#endif
    }
}

static OctreeNode* create_octree_node(int level) {
    OctreeNode* node = calloc(1, sizeof(OctreeNode));
    if (node) {
        node->level = level;
        node->is_leaf = (level == 7); // Leaf at maximum depth
    }
    return node;
}

static void free_octree_node(OctreeNode* node) {
    if (!node) return;
    
    for (int i = 0; i < 8; i++) {
        if (node->children[i]) {
            free_octree_node(node->children[i]);
        }
    }
    free(node);
}

static int get_octree_index(uint8_t r, uint8_t g, uint8_t b, int level) {
    int bit = 7 - level;
    int index = 0;
    if (r & (1 << bit)) index |= 4;
    if (g & (1 << bit)) index |= 2;
    if (b & (1 << bit)) index |= 1;
    return index;
}

static void insert_color_octree(OctreeNode* root, uint8_t r, uint8_t g, uint8_t b) {
    OctreeNode* node = root;
    
    for (int level = 0; level < 8; level++) {
        if (node->is_leaf) {
            node->red_sum += r;
            node->green_sum += g;
            node->blue_sum += b;
            node->pixel_count++;
            return;
        }
        
        int index = get_octree_index(r, g, b, level);
        if (!node->children[index]) {
            node->children[index] = create_octree_node(level + 1);
            if (!node->children[index]) return; // Out of memory
        }
        
        node = node->children[index];
    }
    
    // Reached maximum depth
    node->red_sum += r;
    node->green_sum += g;
    node->blue_sum += b;
    node->pixel_count++;
}

static int collect_palette_colors(OctreeNode* node, Color32* palette, int* count, int max_colors) {
    if (!node) return 0;
    
    if (node->is_leaf || node->pixel_count > 0) {
        if (*count < max_colors && node->pixel_count > 0) {
            palette[*count].r = (uint8_t)(node->red_sum / node->pixel_count);
            palette[*count].g = (uint8_t)(node->green_sum / node->pixel_count);
            palette[*count].b = (uint8_t)(node->blue_sum / node->pixel_count);
            palette[*count].a = 255;
            (*count)++;
        }
        return 1;
    }
    
    for (int i = 0; i < 8; i++) {
        if (node->children[i]) {
            collect_palette_colors(node->children[i], palette, count, max_colors);
            if (*count >= max_colors) break;
        }
    }
    
    return 0;
}

static int find_nearest_color_simd(uint8_t r, uint8_t g, uint8_t b, const Color32* palette, size_t palette_size) {
    detect_simd_features();
    
    if (has_sse && palette_size >= 4) {
        __m128i target = _mm_set_epi32(0, b, g, r);
        int best_index = 0;
        int best_distance = INT_MAX;
        
        size_t simd_size = palette_size & ~3;
        
        for (size_t i = 0; i < simd_size; i += 4) {
            // Load 4 palette colors
            __m128i colors = _mm_set_epi32(
                (palette[i+3].a << 24) | (palette[i+3].b << 16) | (palette[i+3].g << 8) | palette[i+3].r,
                (palette[i+2].a << 24) | (palette[i+2].b << 16) | (palette[i+2].g << 8) | palette[i+2].r,
                (palette[i+1].a << 24) | (palette[i+1].b << 16) | (palette[i+1].g << 8) | palette[i+1].r,
                (palette[i].a << 24) | (palette[i].b << 16) | (palette[i].g << 8) | palette[i].r
            );
            
            // Calculate squared differences (simplified distance)
            for (int j = 0; j < 4 && i + j < palette_size; j++) {
                int dr = r - palette[i+j].r;
                int dg = g - palette[i+j].g;
                int db = b - palette[i+j].b;
                int distance = dr*dr + dg*dg + db*db;
                
                if (distance < best_distance) {
                    best_distance = distance;
                    best_index = i + j;
                }
            }
        }
        
        // Handle remaining colors
        for (size_t i = simd_size; i < palette_size; i++) {
            int dr = r - palette[i].r;
            int dg = g - palette[i].g;
            int db = b - palette[i].b;
            int distance = dr*dr + dg*dg + db*db;
            
            if (distance < best_distance) {
                best_distance = distance;
                best_index = i;
            }
        }
        
        return best_index;
    } else {
        // Fallback scalar implementation
        int best_index = 0;
        int best_distance = INT_MAX;
        
        for (size_t i = 0; i < palette_size; i++) {
            int dr = r - palette[i].r;
            int dg = g - palette[i].g;
            int db = b - palette[i].b;
            int distance = dr*dr + dg*dg + db*db;
            
            if (distance < best_distance) {
                best_distance = distance;
                best_index = i;
            }
        }
        
        return best_index;
    }
}

QuantizedImage* quantize_colors_octree(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    size_t max_colors
) {
    if (!rgba_data || max_colors == 0) return NULL;
    
    // Create result structure
    QuantizedImage* result = malloc(sizeof(QuantizedImage));
    if (!result) return NULL;
    
    result->width = width;
    result->height = height;
    result->palette_size = max_colors > 256 ? 256 : max_colors;
    result->palette = malloc(sizeof(Color32) * result->palette_size);
    result->indices = malloc(width * height);
    
    if (!result->palette || !result->indices) {
        free_quantized_image(result);
        return NULL;
    }
    
    // Build octree
    OctreeNode* root = create_octree_node(0);
    if (!root) {
        free_quantized_image(result);
        return NULL;
    }
    
    // Insert all colors into octree
    for (size_t i = 0; i < width * height; i++) {
        insert_color_octree(root, rgba_data[i*4], rgba_data[i*4+1], rgba_data[i*4+2]);
    }
    
    // Collect palette colors from octree
    int palette_count = 0;
    collect_palette_colors(root, result->palette, &palette_count, result->palette_size);
    result->palette_size = palette_count;
    
    // Map each pixel to nearest palette color
    for (size_t i = 0; i < width * height; i++) {
        result->indices[i] = find_nearest_color_simd(
            rgba_data[i*4], rgba_data[i*4+1], rgba_data[i*4+2],
            result->palette, result->palette_size
        );
    }
    
    free_octree_node(root);
    return result;
}

typedef struct {
    uint8_t r, g, b;
    size_t count;
} ColorBucket;

typedef struct {
    ColorBucket* colors;
    size_t color_count;
    uint8_t min_r, max_r;
    uint8_t min_g, max_g;
    uint8_t min_b, max_b;
} ColorBox;

static int compare_r(const void* a, const void* b) {
    return ((ColorBucket*)a)->r - ((ColorBucket*)b)->r;
}

static int compare_g(const void* a, const void* b) {
    return ((ColorBucket*)a)->g - ((ColorBucket*)b)->g;
}

static int compare_b(const void* a, const void* b) {
    return ((ColorBucket*)a)->b - ((ColorBucket*)b)->b;
}

static void calculate_box_bounds(ColorBox* box) {
    if (box->color_count == 0) return;
    
    box->min_r = box->max_r = box->colors[0].r;
    box->min_g = box->max_g = box->colors[0].g;
    box->min_b = box->max_b = box->colors[0].b;
    
    for (size_t i = 1; i < box->color_count; i++) {
        if (box->colors[i].r < box->min_r) box->min_r = box->colors[i].r;
        if (box->colors[i].r > box->max_r) box->max_r = box->colors[i].r;
        if (box->colors[i].g < box->min_g) box->min_g = box->colors[i].g;
        if (box->colors[i].g > box->max_g) box->max_g = box->colors[i].g;
        if (box->colors[i].b < box->min_b) box->min_b = box->colors[i].b;
        if (box->colors[i].b > box->max_b) box->max_b = box->colors[i].b;
    }
}

static ColorBox split_box(ColorBox* box) {
    ColorBox new_box = {0};
    
    if (box->color_count < 2) return new_box;
    
    // Find the dimension with the largest range
    int r_range = box->max_r - box->min_r;
    int g_range = box->max_g - box->min_g;
    int b_range = box->max_b - box->min_b;
    
    // Sort by the dimension with largest range
    if (r_range >= g_range && r_range >= b_range) {
        qsort(box->colors, box->color_count, sizeof(ColorBucket), compare_r);
    } else if (g_range >= b_range) {
        qsort(box->colors, box->color_count, sizeof(ColorBucket), compare_g);
    } else {
        qsort(box->colors, box->color_count, sizeof(ColorBucket), compare_b);
    }
    
    // Split at median
    size_t median = box->color_count / 2;
    
    new_box.colors = &box->colors[median];
    new_box.color_count = box->color_count - median;
    box->color_count = median;
    
    calculate_box_bounds(box);
    calculate_box_bounds(&new_box);
    
    return new_box;
}

QuantizedImage* quantize_colors_median_cut(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    size_t max_colors
) {
    if (!rgba_data || max_colors == 0) return NULL;
    
    // Create result structure
    QuantizedImage* result = malloc(sizeof(QuantizedImage));
    if (!result) return NULL;
    
    result->width = width;
    result->height = height;
    result->palette_size = max_colors > 256 ? 256 : max_colors;
    result->palette = malloc(sizeof(Color32) * result->palette_size);
    result->indices = malloc(width * height);
    
    if (!result->palette || !result->indices) {
        free_quantized_image(result);
        return NULL;
    }
    
    // Build histogram of unique colors
    size_t max_unique_colors = width * height;
    ColorBucket* unique_colors = malloc(sizeof(ColorBucket) * max_unique_colors);
    if (!unique_colors) {
        free_quantized_image(result);
        return NULL;
    }
    
    size_t unique_count = 0;
    
    // Simple histogram (could be optimized with hash table)
    for (size_t i = 0; i < width * height; i++) {
        uint8_t r = rgba_data[i * 4];
        uint8_t g = rgba_data[i * 4 + 1];
        uint8_t b = rgba_data[i * 4 + 2];
        
        // Find existing color or add new one
        size_t j;
        for (j = 0; j < unique_count; j++) {
            if (unique_colors[j].r == r && unique_colors[j].g == g && unique_colors[j].b == b) {
                unique_colors[j].count++;
                break;
            }
        }
        
        if (j == unique_count && unique_count < max_unique_colors) {
            unique_colors[unique_count].r = r;
            unique_colors[unique_count].g = g;
            unique_colors[unique_count].b = b;
            unique_colors[unique_count].count = 1;
            unique_count++;
        }
    }
    
    // Create initial box containing all colors
    ColorBox* boxes = malloc(sizeof(ColorBox) * result->palette_size);
    if (!boxes) {
        free(unique_colors);
        free_quantized_image(result);
        return NULL;
    }
    
    boxes[0].colors = unique_colors;
    boxes[0].color_count = unique_count;
    calculate_box_bounds(&boxes[0]);
    
    size_t box_count = 1;
    
    // Split boxes until we have enough colors
    while (box_count < result->palette_size && box_count < unique_count) {
        // Find box with largest range to split
        int best_box = -1;
        int best_range = -1;
        
        for (size_t i = 0; i < box_count; i++) {
            if (boxes[i].color_count < 2) continue;
            
            int r_range = boxes[i].max_r - boxes[i].min_r;
            int g_range = boxes[i].max_g - boxes[i].min_g;
            int b_range = boxes[i].max_b - boxes[i].min_b;
            int total_range = r_range + g_range + b_range;
            
            if (total_range > best_range) {
                best_range = total_range;
                best_box = i;
            }
        }
        
        if (best_box == -1) break;
        
        // Split the best box
        boxes[box_count] = split_box(&boxes[best_box]);
        if (boxes[box_count].color_count > 0) {
            box_count++;
        }
    }
    
    // Generate palette from boxes
    result->palette_size = box_count;
    for (size_t i = 0; i < box_count; i++) {
        // Calculate average color for this box
        size_t total_r = 0, total_g = 0, total_b = 0, total_count = 0;
        
        for (size_t j = 0; j < boxes[i].color_count; j++) {
            total_r += boxes[i].colors[j].r * boxes[i].colors[j].count;
            total_g += boxes[i].colors[j].g * boxes[i].colors[j].count;
            total_b += boxes[i].colors[j].b * boxes[i].colors[j].count;
            total_count += boxes[i].colors[j].count;
        }
        
        if (total_count > 0) {
            result->palette[i].r = (uint8_t)(total_r / total_count);
            result->palette[i].g = (uint8_t)(total_g / total_count);
            result->palette[i].b = (uint8_t)(total_b / total_count);
            result->palette[i].a = 255;
        }
    }
    
    // Map each pixel to nearest palette color
    for (size_t i = 0; i < width * height; i++) {
        result->indices[i] = find_nearest_color_simd(
            rgba_data[i * 4], rgba_data[i * 4 + 1], rgba_data[i * 4 + 2],
            result->palette, result->palette_size
        );
    }
    
    free(unique_colors);
    free(boxes);
    return result;
}

void apply_floyd_steinberg_dither(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const Color32* palette,
    size_t palette_size
) {
    if (!rgba_data || !palette || palette_size == 0) return;
    
    // Floyd-Steinberg error diffusion matrix:
    //     X  7/16
    // 3/16 5/16 1/16
    
    int* error_buffer = calloc(width * 3, sizeof(int)); // RGB error buffer for current row
    if (!error_buffer) return;
    
    for (size_t y = 0; y < height; y++) {
        int* next_error = calloc(width * 3, sizeof(int)); // Next row error buffer
        if (!next_error) {
            free(error_buffer);
            return;
        }
        
        for (size_t x = 0; x < width; x++) {
            size_t pixel_idx = (y * width + x) * 4;
            
            // Get current pixel with accumulated error
            int r = (int)rgba_data[pixel_idx] + error_buffer[x * 3];
            int g = (int)rgba_data[pixel_idx + 1] + error_buffer[x * 3 + 1];
            int b = (int)rgba_data[pixel_idx + 2] + error_buffer[x * 3 + 2];
            
            // Clamp values
            r = r < 0 ? 0 : (r > 255 ? 255 : r);
            g = g < 0 ? 0 : (g > 255 ? 255 : g);
            b = b < 0 ? 0 : (b > 255 ? 255 : b);
            
            // Find nearest palette color
            int best_idx = find_nearest_color_simd(r, g, b, palette, palette_size);
            
            // Calculate error
            int err_r = r - palette[best_idx].r;
            int err_g = g - palette[best_idx].g;
            int err_b = b - palette[best_idx].b;
            
            // Update pixel to palette color
            rgba_data[pixel_idx] = palette[best_idx].r;
            rgba_data[pixel_idx + 1] = palette[best_idx].g;
            rgba_data[pixel_idx + 2] = palette[best_idx].b;
            
            // Distribute error to neighboring pixels
            // Right pixel (7/16)
            if (x + 1 < width) {
                error_buffer[(x + 1) * 3] += (err_r * 7) >> 4;
                error_buffer[(x + 1) * 3 + 1] += (err_g * 7) >> 4;
                error_buffer[(x + 1) * 3 + 2] += (err_b * 7) >> 4;
            }
            
            // Next row pixels (3/16, 5/16, 1/16)
            if (y + 1 < height) {
                // Bottom-left (3/16)
                if (x > 0) {
                    next_error[(x - 1) * 3] += (err_r * 3) >> 4;
                    next_error[(x - 1) * 3 + 1] += (err_g * 3) >> 4;
                    next_error[(x - 1) * 3 + 2] += (err_b * 3) >> 4;
                }
                
                // Bottom (5/16)
                next_error[x * 3] += (err_r * 5) >> 4;
                next_error[x * 3 + 1] += (err_g * 5) >> 4;
                next_error[x * 3 + 2] += (err_b * 5) >> 4;
                
                // Bottom-right (1/16)
                if (x + 1 < width) {
                    next_error[(x + 1) * 3] += err_r >> 4;
                    next_error[(x + 1) * 3 + 1] += err_g >> 4;
                    next_error[(x + 1) * 3 + 2] += err_b >> 4;
                }
            }
        }
        
        free(error_buffer);
        error_buffer = next_error;
    }
    
    free(error_buffer);
}

void apply_ordered_dither(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    const Color32* palette,
    size_t palette_size,
    int matrix_size
) {
    if (!rgba_data || !palette || palette_size == 0) return;
    
    // 4x4 Bayer dithering matrix (scaled to 0-255)
    static const int bayer_4x4[16] = {
        0,   128, 32,  160,
        192, 64,  224, 96,
        48,  176, 16,  144,
        240, 112, 208, 80
    };
    
    // 8x8 Bayer dithering matrix (scaled to 0-255)  
    static const int bayer_8x8[64] = {
        0,   128, 32,  160, 8,   136, 40,  168,
        192, 64,  224, 96,  200, 72,  232, 104,
        48,  176, 16,  144, 56,  184, 24,  152,
        240, 112, 208, 80,  248, 120, 216, 88,
        12,  140, 44,  172, 4,   132, 36,  164,
        204, 76,  236, 108, 196, 68,  228, 100,
        60,  188, 28,  156, 52,  180, 20,  148,
        252, 124, 220, 92,  244, 116, 212, 84
    };
    
    const int* dither_matrix;
    int matrix_dim;
    
    if (matrix_size <= 4) {
        dither_matrix = bayer_4x4;
        matrix_dim = 4;
    } else {
        dither_matrix = bayer_8x8;
        matrix_dim = 8;
    }
    
    for (size_t y = 0; y < height; y++) {
        for (size_t x = 0; x < width; x++) {
            size_t pixel_idx = (y * width + x) * 4;
            
            // Get dither threshold from matrix
            int dither_value = dither_matrix[(y % matrix_dim) * matrix_dim + (x % matrix_dim)];
            
            // Apply dithering offset to each channel
            int r = rgba_data[pixel_idx] + ((dither_value - 128) >> 2);
            int g = rgba_data[pixel_idx + 1] + ((dither_value - 128) >> 2);
            int b = rgba_data[pixel_idx + 2] + ((dither_value - 128) >> 2);
            
            // Clamp values
            r = r < 0 ? 0 : (r > 255 ? 255 : r);
            g = g < 0 ? 0 : (g > 255 ? 255 : g);
            b = b < 0 ? 0 : (b > 255 ? 255 : b);
            
            // Find nearest palette color
            int best_idx = find_nearest_color_simd(r, g, b, palette, palette_size);
            
            // Update pixel to palette color
            rgba_data[pixel_idx] = palette[best_idx].r;
            rgba_data[pixel_idx + 1] = palette[best_idx].g;
            rgba_data[pixel_idx + 2] = palette[best_idx].b;
        }
    }
}

void apply_gaussian_blur(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    float sigma
) {
    if (!rgba_data || sigma <= 0.0f) return;
    
    detect_simd_features();
    
    // Calculate kernel radius (3 sigma rule)
    int radius = (int)(sigma * 3.0f + 0.5f);
    if (radius < 1) radius = 1;
    
    int kernel_size = radius * 2 + 1;
    float* kernel = malloc(kernel_size * sizeof(float));
    if (!kernel) return;
    
    // Generate 1D Gaussian kernel
    float sum = 0.0f;
    float sigma_sq_2 = 2.0f * sigma * sigma;
    
    for (int i = 0; i < kernel_size; i++) {
        int x = i - radius;
        kernel[i] = expf(-(x * x) / sigma_sq_2);
        sum += kernel[i];
    }
    
    // Normalize kernel
    for (int i = 0; i < kernel_size; i++) {
        kernel[i] /= sum;
    }
    
    // Allocate temporary buffer
    uint8_t* temp_buffer = malloc(width * height * 4);
    if (!temp_buffer) {
        free(kernel);
        return;
    }
    
    // Horizontal pass
    for (size_t y = 0; y < height; y++) {
        for (size_t x = 0; x < width; x++) {
            float r = 0.0f, g = 0.0f, b = 0.0f, a = 0.0f;
            
            for (int k = 0; k < kernel_size; k++) {
                int sample_x = (int)x + k - radius;
                
                // Handle boundary conditions (clamp)
                if (sample_x < 0) sample_x = 0;
                if (sample_x >= (int)width) sample_x = width - 1;
                
                size_t sample_idx = (y * width + sample_x) * 4;
                float weight = kernel[k];
                
                r += rgba_data[sample_idx] * weight;
                g += rgba_data[sample_idx + 1] * weight;
                b += rgba_data[sample_idx + 2] * weight;
                a += rgba_data[sample_idx + 3] * weight;
            }
            
            size_t pixel_idx = (y * width + x) * 4;
            temp_buffer[pixel_idx] = (uint8_t)(r + 0.5f);
            temp_buffer[pixel_idx + 1] = (uint8_t)(g + 0.5f);
            temp_buffer[pixel_idx + 2] = (uint8_t)(b + 0.5f);
            temp_buffer[pixel_idx + 3] = (uint8_t)(a + 0.5f);
        }
    }
    
    // Vertical pass (with SIMD optimization for SSE)
    if (has_sse) {
        for (size_t x = 0; x < width; x++) {
            for (size_t y = 0; y < height; y++) {
                __m128 result = _mm_setzero_ps();
                
                for (int k = 0; k < kernel_size; k++) {
                    int sample_y = (int)y + k - radius;
                    
                    // Handle boundary conditions (clamp)
                    if (sample_y < 0) sample_y = 0;
                    if (sample_y >= (int)height) sample_y = height - 1;
                    
                    size_t sample_idx = (sample_y * width + x) * 4;
                    
                    // Load RGBA as floats
                    __m128 pixel = _mm_set_ps(
                        temp_buffer[sample_idx + 3],
                        temp_buffer[sample_idx + 2],
                        temp_buffer[sample_idx + 1],
                        temp_buffer[sample_idx]
                    );
                    
                    __m128 weight = _mm_set1_ps(kernel[k]);
                    result = _mm_add_ps(result, _mm_mul_ps(pixel, weight));
                }
                
                // Store result
                size_t pixel_idx = (y * width + x) * 4;
                float pixel_data[4];
                _mm_store_ps(pixel_data, result);
                
                rgba_data[pixel_idx] = (uint8_t)(pixel_data[0] + 0.5f);
                rgba_data[pixel_idx + 1] = (uint8_t)(pixel_data[1] + 0.5f);
                rgba_data[pixel_idx + 2] = (uint8_t)(pixel_data[2] + 0.5f);
                rgba_data[pixel_idx + 3] = (uint8_t)(pixel_data[3] + 0.5f);
            }
        }
    } else {
        // Fallback scalar vertical pass
        for (size_t x = 0; x < width; x++) {
            for (size_t y = 0; y < height; y++) {
                float r = 0.0f, g = 0.0f, b = 0.0f, a = 0.0f;
                
                for (int k = 0; k < kernel_size; k++) {
                    int sample_y = (int)y + k - radius;
                    
                    // Handle boundary conditions (clamp)
                    if (sample_y < 0) sample_y = 0;
                    if (sample_y >= (int)height) sample_y = height - 1;
                    
                    size_t sample_idx = (sample_y * width + x) * 4;
                    float weight = kernel[k];
                    
                    r += temp_buffer[sample_idx] * weight;
                    g += temp_buffer[sample_idx + 1] * weight;
                    b += temp_buffer[sample_idx + 2] * weight;
                    a += temp_buffer[sample_idx + 3] * weight;
                }
                
                size_t pixel_idx = (y * width + x) * 4;
                rgba_data[pixel_idx] = (uint8_t)(r + 0.5f);
                rgba_data[pixel_idx + 1] = (uint8_t)(g + 0.5f);
                rgba_data[pixel_idx + 2] = (uint8_t)(b + 0.5f);
                rgba_data[pixel_idx + 3] = (uint8_t)(a + 0.5f);
            }
        }
    }
    
    free(kernel);
    free(temp_buffer);
}

void apply_sharpen_filter(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    float strength
) {
    if (!rgba_data || strength <= 0.0f) return;
    
    detect_simd_features();
    
    // Unsharp mask kernel (center weight = 1 + 4*strength, neighbors = -strength)
    float center_weight = 1.0f + 4.0f * strength;
    float neighbor_weight = -strength;
    
    uint8_t* temp_buffer = malloc(width * height * 4);
    if (!temp_buffer) return;
    
    memcpy(temp_buffer, rgba_data, width * height * 4);
    
    if (has_sse) {
        __m128 center_vec = _mm_set1_ps(center_weight);
        __m128 neighbor_vec = _mm_set1_ps(neighbor_weight);
        
        for (size_t y = 1; y < height - 1; y++) {
            for (size_t x = 1; x < width - 1; x++) {
                size_t center_idx = (y * width + x) * 4;
                
                // Load center pixel
                __m128 center = _mm_set_ps(
                    temp_buffer[center_idx + 3],
                    temp_buffer[center_idx + 2],
                    temp_buffer[center_idx + 1],
                    temp_buffer[center_idx]
                );
                
                // Apply center weight
                __m128 result = _mm_mul_ps(center, center_vec);
                
                // Add neighbor contributions
                size_t neighbor_indices[4] = {
                    ((y-1) * width + x) * 4,     // Top
                    ((y+1) * width + x) * 4,     // Bottom
                    (y * width + (x-1)) * 4,     // Left
                    (y * width + (x+1)) * 4      // Right
                };
                
                for (int i = 0; i < 4; i++) {
                    __m128 neighbor = _mm_set_ps(
                        temp_buffer[neighbor_indices[i] + 3],
                        temp_buffer[neighbor_indices[i] + 2],
                        temp_buffer[neighbor_indices[i] + 1],
                        temp_buffer[neighbor_indices[i]]
                    );
                    
                    result = _mm_add_ps(result, _mm_mul_ps(neighbor, neighbor_vec));
                }
                
                // Store result with clamping
                float pixel_data[4];
                _mm_store_ps(pixel_data, result);
                
                for (int c = 0; c < 4; c++) {
                    float value = pixel_data[c];
                    rgba_data[center_idx + c] = (uint8_t)(value < 0 ? 0 : (value > 255 ? 255 : value + 0.5f));
                }
            }
        }
    } else {
        // Fallback scalar implementation
        for (size_t y = 1; y < height - 1; y++) {
            for (size_t x = 1; x < width - 1; x++) {
                size_t center_idx = (y * width + x) * 4;
                
                for (int c = 0; c < 4; c++) {
                    float result = temp_buffer[center_idx + c] * center_weight;
                    
                    // Add neighbor contributions
                    result += temp_buffer[((y-1) * width + x) * 4 + c] * neighbor_weight;  // Top
                    result += temp_buffer[((y+1) * width + x) * 4 + c] * neighbor_weight;  // Bottom
                    result += temp_buffer[(y * width + (x-1)) * 4 + c] * neighbor_weight;  // Left
                    result += temp_buffer[(y * width + (x+1)) * 4 + c] * neighbor_weight;  // Right
                    
                    // Clamp and store
                    rgba_data[center_idx + c] = (uint8_t)(result < 0 ? 0 : (result > 255 ? 255 : result + 0.5f));
                }
            }
        }
    }
    
    free(temp_buffer);
}

void apply_edge_detection(
    uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t* output
) {
    if (!rgba_data || !output) return;
    
    detect_simd_features();
    
    // Sobel edge detection kernels
    static const int sobel_x[9] = {
        -1, 0, 1,
        -2, 0, 2,
        -1, 0, 1
    };
    
    static const int sobel_y[9] = {
        -1, -2, -1,
         0,  0,  0,
         1,  2,  1
    };
    
    // Convert to grayscale first for edge detection
    uint8_t* gray = malloc(width * height);
    if (!gray) return;
    
    if (has_sse) {
        // SIMD grayscale conversion
        __m128 luma_weights = _mm_set_ps(0.0f, 0.114f, 0.587f, 0.299f);
        
        for (size_t i = 0; i < width * height; i++) {
            __m128 rgba = _mm_set_ps(
                rgba_data[i*4 + 3],
                rgba_data[i*4 + 2],
                rgba_data[i*4 + 1],
                rgba_data[i*4]
            );
            
            __m128 weighted = _mm_mul_ps(rgba, luma_weights);
            
            // Horizontal sum
            weighted = _mm_hadd_ps(weighted, weighted);
            weighted = _mm_hadd_ps(weighted, weighted);
            
            gray[i] = (uint8_t)(_mm_cvtss_f32(weighted) + 0.5f);
        }
    } else {
        // Scalar grayscale conversion
        for (size_t i = 0; i < width * height; i++) {
            gray[i] = (uint8_t)(0.299f * rgba_data[i*4] + 
                               0.587f * rgba_data[i*4+1] + 
                               0.114f * rgba_data[i*4+2] + 0.5f);
        }
    }
    
    // Apply Sobel edge detection
    for (size_t y = 1; y < height - 1; y++) {
        for (size_t x = 1; x < width - 1; x++) {
            int gx = 0, gy = 0;
            
            // Convolve with Sobel kernels
            for (int ky = -1; ky <= 1; ky++) {
                for (int kx = -1; kx <= 1; kx++) {
                    int pixel_val = gray[(y + ky) * width + (x + kx)];
                    int kernel_idx = (ky + 1) * 3 + (kx + 1);
                    
                    gx += pixel_val * sobel_x[kernel_idx];
                    gy += pixel_val * sobel_y[kernel_idx];
                }
            }
            
            // Calculate gradient magnitude
            int magnitude = (int)(sqrtf(gx * gx + gy * gy) + 0.5f);
            output[y * width + x] = (uint8_t)(magnitude > 255 ? 255 : magnitude);
        }
    }
    
    // Handle borders (set to 0)
    for (size_t x = 0; x < width; x++) {
        output[x] = 0;                           // Top row
        output[(height - 1) * width + x] = 0;    // Bottom row
    }
    for (size_t y = 0; y < height; y++) {
        output[y * width] = 0;                   // Left column
        output[y * width + (width - 1)] = 0;     // Right column
    }
    
    free(gray);
}

void rgb_to_yuv(const uint8_t* rgb, uint8_t* yuv, size_t pixel_count) {
    if (!rgb || !yuv) return;
    
    detect_simd_features();
    
    if (has_sse && pixel_count >= 4) {
        // ITU-R BT.709 conversion matrix (scaled for integer math)
        __m128 y_weights = _mm_set_ps(0.0f, 0.0722f, 0.7152f, 0.2126f);
        __m128 u_weights = _mm_set_ps(0.0f, 0.5389f, -0.3959f, -0.1430f);
        __m128 v_weights = _mm_set_ps(0.0f, -0.0458f, -0.5142f, 0.5600f);
        __m128 offset_128 = _mm_set1_ps(128.0f);
        
        size_t simd_count = pixel_count & ~3;
        
        for (size_t i = 0; i < simd_count; i += 4) {
            // Process 4 pixels at once
            for (int p = 0; p < 4; p++) {
                __m128 rgb_pixel = _mm_set_ps(
                    0.0f,
                    rgb[(i + p) * 3 + 2],  // B
                    rgb[(i + p) * 3 + 1],  // G
                    rgb[(i + p) * 3]       // R
                );
                
                // Calculate Y, U, V
                __m128 y_val = _mm_dp_ps(rgb_pixel, y_weights, 0x71);
                __m128 u_val = _mm_add_ps(_mm_dp_ps(rgb_pixel, u_weights, 0x71), offset_128);
                __m128 v_val = _mm_add_ps(_mm_dp_ps(rgb_pixel, v_weights, 0x71), offset_128);
                
                yuv[(i + p) * 3] = (uint8_t)(_mm_cvtss_f32(y_val) + 0.5f);
                yuv[(i + p) * 3 + 1] = (uint8_t)(_mm_cvtss_f32(u_val) + 0.5f);
                yuv[(i + p) * 3 + 2] = (uint8_t)(_mm_cvtss_f32(v_val) + 0.5f);
            }
        }
        
        // Handle remaining pixels
        for (size_t i = simd_count; i < pixel_count; i++) {
            uint8_t r = rgb[i * 3];
            uint8_t g = rgb[i * 3 + 1];
            uint8_t b = rgb[i * 3 + 2];
            
            yuv[i * 3] = (uint8_t)(0.2126f * r + 0.7152f * g + 0.0722f * b + 0.5f);
            yuv[i * 3 + 1] = (uint8_t)(-0.1430f * r - 0.3959f * g + 0.5389f * b + 128.5f);
            yuv[i * 3 + 2] = (uint8_t)(0.5600f * r - 0.5142f * g - 0.0458f * b + 128.5f);
        }
    } else {
        // Scalar fallback - ITU-R BT.709 conversion
        for (size_t i = 0; i < pixel_count; i++) {
            uint8_t r = rgb[i * 3];
            uint8_t g = rgb[i * 3 + 1];
            uint8_t b = rgb[i * 3 + 2];
            
            // Y (luma)
            yuv[i * 3] = (uint8_t)(0.2126f * r + 0.7152f * g + 0.0722f * b + 0.5f);
            
            // U (Cb) 
            yuv[i * 3 + 1] = (uint8_t)(-0.1430f * r - 0.3959f * g + 0.5389f * b + 128.5f);
            
            // V (Cr)
            yuv[i * 3 + 2] = (uint8_t)(0.5600f * r - 0.5142f * g - 0.0458f * b + 128.5f);
        }
    }
}

void yuv_to_rgb(const uint8_t* yuv, uint8_t* rgb, size_t pixel_count) {
    if (!yuv || !rgb) return;
    
    detect_simd_features();
    
    if (has_sse && pixel_count >= 4) {
        __m128 offset_128 = _mm_set1_ps(128.0f);
        
        for (size_t i = 0; i < pixel_count; i++) {
            float y = yuv[i * 3];
            float u = yuv[i * 3 + 1] - 128.0f;
            float v = yuv[i * 3 + 2] - 128.0f;
            
            // ITU-R BT.709 YUV to RGB conversion
            __m128 y_vec = _mm_set1_ps(y);
            __m128 u_vec = _mm_set1_ps(u);
            __m128 v_vec = _mm_set1_ps(v);
            
            // R = Y + 1.5748 * V
            // G = Y - 0.1873 * U - 0.4681 * V  
            // B = Y + 1.8556 * U
            
            __m128 r_val = _mm_add_ps(y_vec, _mm_mul_ps(v_vec, _mm_set1_ps(1.5748f)));
            __m128 g_val = _mm_sub_ps(y_vec, _mm_add_ps(
                _mm_mul_ps(u_vec, _mm_set1_ps(0.1873f)),
                _mm_mul_ps(v_vec, _mm_set1_ps(0.4681f))
            ));
            __m128 b_val = _mm_add_ps(y_vec, _mm_mul_ps(u_vec, _mm_set1_ps(1.8556f)));
            
            // Clamp and convert to uint8
            float r_result = _mm_cvtss_f32(r_val);
            float g_result = _mm_cvtss_f32(g_val);
            float b_result = _mm_cvtss_f32(b_val);
            
            rgb[i * 3] = (uint8_t)(r_result < 0 ? 0 : (r_result > 255 ? 255 : r_result + 0.5f));
            rgb[i * 3 + 1] = (uint8_t)(g_result < 0 ? 0 : (g_result > 255 ? 255 : g_result + 0.5f));
            rgb[i * 3 + 2] = (uint8_t)(b_result < 0 ? 0 : (b_result > 255 ? 255 : b_result + 0.5f));
        }
    } else {
        // Scalar fallback - ITU-R BT.709 conversion
        for (size_t i = 0; i < pixel_count; i++) {
            float y = yuv[i * 3];
            float u = yuv[i * 3 + 1] - 128.0f;
            float v = yuv[i * 3 + 2] - 128.0f;
            
            // ITU-R BT.709 YUV to RGB conversion
            float r = y + 1.5748f * v;
            float g = y - 0.1873f * u - 0.4681f * v;
            float b = y + 1.8556f * u;
            
            // Clamp and convert to uint8
            rgb[i * 3] = (uint8_t)(r < 0 ? 0 : (r > 255 ? 255 : r + 0.5f));
            rgb[i * 3 + 1] = (uint8_t)(g < 0 ? 0 : (g > 255 ? 255 : g + 0.5f));
            rgb[i * 3 + 2] = (uint8_t)(b < 0 ? 0 : (b > 255 ? 255 : b + 0.5f));
        }
    }
}

static float f_xyz_helper(float t) {
    const float delta = 6.0f / 29.0f;
    if (t > delta * delta * delta) {
        return powf(t, 1.0f / 3.0f);
    } else {
        return t / (3.0f * delta * delta) + 4.0f / 29.0f;
    }
}

void rgb_to_lab(const uint8_t* rgb, float* lab, size_t pixel_count) {
    if (!rgb || !lab) return;
    
    // XYZ white point (D65 illuminant)
    const float xn = 95.047f;
    const float yn = 100.0f;
    const float zn = 108.883f;
    
    for (size_t i = 0; i < pixel_count; i++) {
        // Convert RGB to linear RGB (gamma correction)
        float r = rgb[i * 3] / 255.0f;
        float g = rgb[i * 3 + 1] / 255.0f;
        float b = rgb[i * 3 + 2] / 255.0f;
        
        // Apply gamma correction (sRGB to linear)
        r = (r <= 0.04045f) ? r / 12.92f : powf((r + 0.055f) / 1.055f, 2.4f);
        g = (g <= 0.04045f) ? g / 12.92f : powf((g + 0.055f) / 1.055f, 2.4f);
        b = (b <= 0.04045f) ? b / 12.92f : powf((b + 0.055f) / 1.055f, 2.4f);
        
        // Convert RGB to XYZ using sRGB matrix
        float x = 0.4124564f * r + 0.3575761f * g + 0.1804375f * b;
        float y = 0.2126729f * r + 0.7151522f * g + 0.0721750f * b;
        float z = 0.0193339f * r + 0.1191920f * g + 0.9503041f * b;
        
        // Scale to 0-100 range
        x *= 100.0f;
        y *= 100.0f;
        z *= 100.0f;
        
        // Normalize by white point
        float fx = f_xyz_helper(x / xn);
        float fy = f_xyz_helper(y / yn);
        float fz = f_xyz_helper(z / zn);
        
        // Calculate Lab values
        lab[i * 3] = 116.0f * fy - 16.0f;          // L*
        lab[i * 3 + 1] = 500.0f * (fx - fy);       // a*
        lab[i * 3 + 2] = 200.0f * (fy - fz);       // b*
    }
}

static float f_xyz_inv_helper(float t) {
    const float delta = 6.0f / 29.0f;
    if (t > delta) {
        return t * t * t;
    } else {
        return 3.0f * delta * delta * (t - 4.0f / 29.0f);
    }
}

void lab_to_rgb(const float* lab, uint8_t* rgb, size_t pixel_count) {
    if (!lab || !rgb) return;
    
    // XYZ white point (D65 illuminant)
    const float xn = 95.047f;
    const float yn = 100.0f;
    const float zn = 108.883f;
    
    for (size_t i = 0; i < pixel_count; i++) {
        float l = lab[i * 3];
        float a = lab[i * 3 + 1];
        float b = lab[i * 3 + 2];
        
        // Convert Lab to XYZ
        float fy = (l + 16.0f) / 116.0f;
        float fx = a / 500.0f + fy;
        float fz = fy - b / 200.0f;
        
        float x = xn * f_xyz_inv_helper(fx);
        float y = yn * f_xyz_inv_helper(fy);
        float z = zn * f_xyz_inv_helper(fz);
        
        // Scale from 0-100 range
        x /= 100.0f;
        y /= 100.0f;
        z /= 100.0f;
        
        // Convert XYZ to linear RGB using sRGB matrix
        float r = 3.2404542f * x - 1.5371385f * y - 0.4985314f * z;
        float g = -0.9692660f * x + 1.8760108f * y + 0.0415560f * z;
        float b_val = 0.0556434f * x - 0.2040259f * y + 1.0572252f * z;
        
        // Apply gamma correction (linear to sRGB)
        r = (r <= 0.0031308f) ? 12.92f * r : 1.055f * powf(r, 1.0f / 2.4f) - 0.055f;
        g = (g <= 0.0031308f) ? 12.92f * g : 1.055f * powf(g, 1.0f / 2.4f) - 0.055f;
        b_val = (b_val <= 0.0031308f) ? 12.92f * b_val : 1.055f * powf(b_val, 1.0f / 2.4f) - 0.055f;
        
        // Clamp and convert to uint8
        rgb[i * 3] = (uint8_t)((r < 0 ? 0 : (r > 1 ? 1 : r)) * 255.0f + 0.5f);
        rgb[i * 3 + 1] = (uint8_t)((g < 0 ? 0 : (g > 1 ? 1 : g)) * 255.0f + 0.5f);
        rgb[i * 3 + 2] = (uint8_t)((b_val < 0 ? 0 : (b_val > 1 ? 1 : b_val)) * 255.0f + 0.5f);
    }
}

void free_quantized_image(QuantizedImage* img) {
    if (img) {
        free(img->palette);
        free(img->indices);
        free(img);
    }
}
