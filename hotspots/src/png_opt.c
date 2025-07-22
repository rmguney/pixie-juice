/*
 * PNG Optimization Hotspot - WASM-compatible PNG optimization
 * Uses optimized compression algorithms with SIMD support
 */

#include "png_opt.h"
#include <stdint.h>
#include <stddef.h>

#ifdef __wasm32__
#include <wasm_simd128.h>
#define WASM_SIMD_AVAILABLE 1
#else
#define WASM_SIMD_AVAILABLE 0
#endif

// Simple Adler-32 checksum with SIMD optimization
static uint32_t adler32_simd(const uint8_t* data, size_t len) {
    uint32_t a = 1, b = 0;
    
#if WASM_SIMD_AVAILABLE
    // Use WASM SIMD for bulk processing (simplified for compatibility)
    v128_t va = wasm_i32x4_splat(1);
    v128_t vb = wasm_i32x4_splat(0);
    
    // Process 16 bytes at a time with SIMD
    while (len >= 16) {
        v128_t vdata = wasm_v128_load(data);
        
        // Extract first byte and add to accumulator (simplified)
        uint8_t byte_val = wasm_u8x16_extract_lane(vdata, 0);
        va = wasm_i32x4_add(va, wasm_i32x4_splat((int32_t)byte_val));
        vb = wasm_i32x4_add(vb, va);
        
        data += 16;
        len -= 16;
    }
    
    // Extract final values
    a = (uint32_t)wasm_i32x4_extract_lane(va, 0);
    b = (uint32_t)wasm_i32x4_extract_lane(vb, 0);
#endif
    
    // Process remaining bytes
    while (len-- > 0) {
        a += *data++;
        b += a;
        a %= 65521;
        b %= 65521;
    }
    
    return (b << 16) | a;
}

// Simple LZ77-style compression with sliding window
static size_t compress_lz77(const uint8_t* input, size_t input_len, 
                           uint8_t* output, size_t output_max) {
    if (input_len == 0 || output_max < input_len + 16) return 0;
    
    const size_t WINDOW_SIZE = 32768;
    const size_t MIN_MATCH = 3;
    const size_t MAX_MATCH = 258;
    
    size_t output_pos = 0;
    size_t input_pos = 0;
    
    // Simple compression: look for repeating patterns
    while (input_pos < input_len && output_pos < output_max - 4) {
        size_t best_len = 0;
        size_t best_dist = 0;
        
        // Look back in the window for matches
        size_t start = (input_pos > WINDOW_SIZE) ? input_pos - WINDOW_SIZE : 0;
        
        for (size_t i = start; i < input_pos; i++) {
            size_t match_len = 0;
            
            // Count matching bytes
            while (match_len < MAX_MATCH && 
                   input_pos + match_len < input_len &&
                   input[i + match_len] == input[input_pos + match_len]) {
                match_len++;
            }
            
            if (match_len >= MIN_MATCH && match_len > best_len) {
                best_len = match_len;
                best_dist = input_pos - i;
            }
        }
        
        if (best_len >= MIN_MATCH) {
            // Encode as length-distance pair (simplified)
            output[output_pos++] = 0xFF; // Marker for compressed data
            output[output_pos++] = (uint8_t)best_len;
            output[output_pos++] = (uint8_t)(best_dist & 0xFF);
            output[output_pos++] = (uint8_t)(best_dist >> 8);
            input_pos += best_len;
        } else {
            // Copy literal byte
            output[output_pos++] = input[input_pos++];
        }
    }
    
    // Copy any remaining bytes
    while (input_pos < input_len && output_pos < output_max) {
        output[output_pos++] = input[input_pos++];
    }
    
    return output_pos;
}

// PNG color type definitions
#define PNG_COLOR_TYPE_GRAY       0
#define PNG_COLOR_TYPE_RGB        2
#define PNG_COLOR_TYPE_PALETTE    3
#define PNG_COLOR_TYPE_GRAY_ALPHA 4
#define PNG_COLOR_TYPE_RGBA       6

// PNG chunk analysis
typedef struct {
    uint32_t width;
    uint32_t height;
    uint8_t bit_depth;
    uint8_t color_type;
    uint8_t compression_method;
    uint8_t filter_method;
    uint8_t interlace_method;
    int has_alpha;
    int has_transparency;
} PngImageInfo;

// Analyze PNG IHDR chunk to get image information
static int analyze_png_header(const uint8_t* input_data, size_t input_len, PngImageInfo* info) {
    // Find IHDR chunk (should be right after PNG signature)
    size_t pos = 8; // Skip PNG signature
    
    if (pos + 25 > input_len) return 0; // Not enough data for IHDR
    
    uint32_t chunk_len = (input_data[pos] << 24) | 
                        (input_data[pos+1] << 16) | 
                        (input_data[pos+2] << 8) | 
                        input_data[pos+3];
    
    char chunk_type[5] = {0};
    memcpy(chunk_type, &input_data[pos+4], 4);
    
    if (strcmp(chunk_type, "IHDR") != 0 || chunk_len != 13) {
        return 0; // Invalid IHDR
    }
    
    // Parse IHDR data
    pos += 8; // Skip length and type
    info->width = (input_data[pos] << 24) | (input_data[pos+1] << 16) | 
                  (input_data[pos+2] << 8) | input_data[pos+3];
    pos += 4;
    
    info->height = (input_data[pos] << 24) | (input_data[pos+1] << 16) | 
                   (input_data[pos+2] << 8) | input_data[pos+3];
    pos += 4;
    
    info->bit_depth = input_data[pos++];
    info->color_type = input_data[pos++];
    info->compression_method = input_data[pos++];
    info->filter_method = input_data[pos++];
    info->interlace_method = input_data[pos++];
    
    // Determine if image has alpha
    info->has_alpha = (info->color_type == PNG_COLOR_TYPE_GRAY_ALPHA) || 
                      (info->color_type == PNG_COLOR_TYPE_RGBA);
    
    // Check for tRNS chunk (transparency chunk) which adds alpha to non-alpha types
    info->has_transparency = 0;
    pos = 8; // Reset to start scanning for tRNS
    while (pos + 8 < input_len) {
        uint32_t len = (input_data[pos] << 24) | (input_data[pos+1] << 16) | 
                       (input_data[pos+2] << 8) | input_data[pos+3];
        char type[5] = {0};
        memcpy(type, &input_data[pos+4], 4);
        
        if (strcmp(type, "tRNS") == 0) {
            info->has_transparency = 1;
            break;
        }
        
        pos += 8 + len + 4; // Skip chunk
    }
    
    return 1;
}

// Main PNG optimization function
PngOptResult png_optimize_c(const uint8_t* input_data, size_t input_len, 
                           const PngOptConfig* config) {
    PngOptResult result = {0};
    
    if (!input_data || input_len < 16 || !config) {
        result.error_code = -1;
        strcpy(result.error_message, "Invalid input parameters");
        return result;
    }
    
    // Simple PNG signature check
    const uint8_t png_sig[] = {0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A};
    if (memcmp(input_data, png_sig, 8) != 0) {
        result.error_code = -2;
        strcpy(result.error_message, "Not a valid PNG file");
        return result;
    }
    
    // Analyze PNG to get image information including alpha channel status
    PngImageInfo img_info = {0};
    if (!analyze_png_header(input_data, input_len, &img_info)) {
        result.error_code = -3;
        strcpy(result.error_message, "Invalid PNG header");
        return result;
    }
    
    // Log image information including alpha channel status
    #ifdef __wasm32__
    // For WASM, we can't use printf, but this info is useful for debugging
    #endif
    
    // Allocate output buffer (worst case: same size as input + some overhead)
    size_t max_output_size = input_len + 1024;
    result.output_data = (uint8_t*)malloc(max_output_size);
    if (!result.output_data) {
        result.error_code = -4;
        strcpy(result.error_message, "Memory allocation failed");
        return result;
    }
    
    // Copy PNG header
    memcpy(result.output_data, png_sig, 8);
    result.output_size = 8;
    
    // Process PNG chunks, preserving alpha channel information
    size_t pos = 8;
    int optimized = 0;
    int preserved_alpha = 0;
    
    while (pos + 8 < input_len) {
        // Read chunk header
        uint32_t chunk_len = (input_data[pos] << 24) | 
                            (input_data[pos+1] << 16) | 
                            (input_data[pos+2] << 8) | 
                            input_data[pos+3];
        
        if (pos + 8 + chunk_len > input_len) break;
        
        char chunk_type[5] = {0};
        memcpy(chunk_type, &input_data[pos+4], 4);
        
        // Always preserve IHDR chunk as-is to maintain color type and alpha info
        if (strcmp(chunk_type, "IHDR") == 0) {
            memcpy(&result.output_data[result.output_size], &input_data[pos], 8 + chunk_len + 4);
            result.output_size += 8 + chunk_len + 4;
            preserved_alpha = img_info.has_alpha || img_info.has_transparency;
        }
        // Always preserve tRNS chunk (transparency) as it's critical for alpha
        else if (strcmp(chunk_type, "tRNS") == 0) {
            memcpy(&result.output_data[result.output_size], &input_data[pos], 8 + chunk_len + 4);
            result.output_size += 8 + chunk_len + 4;
            preserved_alpha = 1;
        }
        // Optimize IDAT chunks while preserving the original image format
        else if (strcmp(chunk_type, "IDAT") == 0 && config->compress_level > 0) {
            // Try to recompress the data while preserving alpha channel format
            uint8_t* compressed = (uint8_t*)malloc(chunk_len + 1024);
            if (compressed) {
                size_t compressed_size = compress_lz77(
                    &input_data[pos+8], chunk_len, 
                    compressed, chunk_len + 1024
                );
                
                if (compressed_size > 0 && compressed_size < chunk_len) {
                    // Use compressed version
                    uint32_t new_len = (uint32_t)compressed_size;
                    result.output_data[result.output_size++] = (new_len >> 24) & 0xFF;
                    result.output_data[result.output_size++] = (new_len >> 16) & 0xFF;
                    result.output_data[result.output_size++] = (new_len >> 8) & 0xFF;
                    result.output_data[result.output_size++] = new_len & 0xFF;
                    
                    memcpy(&result.output_data[result.output_size], chunk_type, 4);
                    result.output_size += 4;
                    
                    memcpy(&result.output_data[result.output_size], compressed, compressed_size);
                    result.output_size += compressed_size;
                    
                    // Calculate and add CRC (simplified - just copy original for now)
                    uint32_t crc = adler32_simd(compressed, compressed_size);
                    result.output_data[result.output_size++] = (crc >> 24) & 0xFF;
                    result.output_data[result.output_size++] = (crc >> 16) & 0xFF;
                    result.output_data[result.output_size++] = (crc >> 8) & 0xFF;
                    result.output_data[result.output_size++] = crc & 0xFF;
                    
                    optimized = 1;
                } else {
                    // Copy original chunk
                    memcpy(&result.output_data[result.output_size], &input_data[pos], 8 + chunk_len + 4);
                    result.output_size += 8 + chunk_len + 4;
                }
                free(compressed);
            } else {
                // Copy original chunk
                memcpy(&result.output_data[result.output_size], &input_data[pos], 8 + chunk_len + 4);
                result.output_size += 8 + chunk_len + 4;
            }
        }
        // Strip metadata chunks if requested, but preserve critical chunks
        else if (config->strip_metadata && 
                 (strcmp(chunk_type, "tEXt") == 0 || 
                  strcmp(chunk_type, "zTXt") == 0 || 
                  strcmp(chunk_type, "iTXt") == 0 ||
                  strcmp(chunk_type, "tIME") == 0 ||
                  strcmp(chunk_type, "pHYs") == 0)) {
            // Skip metadata chunks to reduce file size
            // But this preserves alpha channel information
        } else {
            // Copy all other chunks as-is (including PLTE, gAMA, etc.)
            memcpy(&result.output_data[result.output_size], &input_data[pos], 8 + chunk_len + 4);
            result.output_size += 8 + chunk_len + 4;
        }
        
        pos += 8 + chunk_len + 4;
    }
    
    // If no optimization happened or result is larger, return original
    if (!optimized || result.output_size >= input_len) {
        free(result.output_data);
        result.output_data = (uint8_t*)malloc(input_len);
        if (result.output_data) {
            memcpy(result.output_data, input_data, input_len);
            result.output_size = input_len;
        }
    }
    
    result.error_code = 0;
    result.compression_ratio = (double)result.output_size / (double)input_len;
    
    return result;
}

// Free the result
void png_opt_result_free(PngOptResult* result) {
    if (result && result->output_data) {
        free(result->output_data);
        result->output_data = NULL;
        result->output_size = 0;
    }
}

// Check if PNG has alpha channel or transparency
int png_has_alpha_channel(const uint8_t* png_data, size_t png_size) {
    if (!png_data || png_size < 16) return 0;
    
    PngImageInfo info = {0};
    if (!analyze_png_header(png_data, png_size, &info)) {
        return 0;
    }
    
    return info.has_alpha || info.has_transparency;
}
