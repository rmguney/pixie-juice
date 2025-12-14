#ifndef COMPRESS_H
#define COMPRESS_H

#include "memory.h"
#include "util.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    METHOD_NONE = 0,
    METHOD_LZ4 = 1,
    METHOD_HUFFMAN = 2,
    METHOD_DEFLATE = 3
} CompressionMethod;

typedef struct {
    uint8_t* data;
    size_t size;
    size_t capacity;
} CompressBuffer;

WASM_EXPORT size_t deflate_compress(
    const uint8_t* input,
    size_t input_size,
    uint8_t* output,
    size_t output_capacity,
    int compression_level,
    int window_bits,
    int mem_level
);

WASM_EXPORT size_t png_compress_scanlines(
    const uint8_t* rgba_data,
    size_t width,
    size_t height,
    uint8_t* compressed_output,
    size_t output_capacity,
    int filter_strategy
);

WASM_EXPORT size_t lz4_compress_fast(
    const uint8_t* input,
    size_t input_size,
    uint8_t* output,
    size_t output_capacity,
    int acceleration
);

WASM_EXPORT size_t lz4_decompress_fast(
    const uint8_t* input,
    size_t input_size,
    uint8_t* output,
    size_t output_capacity
);

WASM_EXPORT size_t zstd_compress_advanced(
    const uint8_t* input,
    size_t input_size,
    uint8_t* output,
    size_t output_capacity,
    int compression_level,
    int window_log,
    int hash_log,
    int chain_log
);

WASM_EXPORT size_t zstd_decompress(
    const uint8_t* input,
    size_t input_size,
    uint8_t* output,
    size_t output_capacity
);

typedef struct {
    uint16_t symbol;
    uint32_t frequency;
    uint8_t code_length;
    uint32_t code;
} HuffmanEntry;

typedef struct {
    HuffmanEntry* entries;
    size_t entry_count;
    uint8_t max_code_length;
} HuffmanTable;

WASM_EXPORT HuffmanTable* build_huffman_table(const uint32_t* frequencies, size_t symbol_count);
WASM_EXPORT size_t huffman_encode(const uint8_t* input, size_t input_size, 
                     const HuffmanTable* table, uint8_t* output, size_t output_capacity);
WASM_EXPORT size_t huffman_decode(const uint8_t* input, size_t input_size,
                     const HuffmanTable* table, uint8_t* output, size_t output_capacity);
WASM_EXPORT void free_huffman_table(HuffmanTable* table);

typedef struct {
    uint8_t* dictionary;
    size_t dictionary_size;
    uint32_t* hash_table;
    size_t hash_table_size;
} DictionaryCompressor;

// Missing functions from build.rs bindings
WASM_EXPORT int compress_lz4(const uint8_t* input, size_t input_size, uint8_t* output, size_t max_output_size);
WASM_EXPORT int decompress_lz4(const uint8_t* input, size_t input_size, uint8_t* output, size_t output_size);
WASM_EXPORT int compress_huffman(const uint8_t* input, size_t input_size, uint8_t* output, size_t max_output_size);
WASM_EXPORT uint32_t get_optimal_compression(const uint8_t* data, size_t size);

DictionaryCompressor* create_dictionary_compressor(size_t dictionary_size, size_t hash_size);
void train_dictionary(DictionaryCompressor* compressor, const uint8_t* training_data, size_t data_size);
size_t dictionary_compress(DictionaryCompressor* compressor, const uint8_t* input, size_t input_size,
                          uint8_t* output, size_t output_capacity);
size_t dictionary_decompress(DictionaryCompressor* compressor, const uint8_t* input, size_t input_size,
                            uint8_t* output, size_t output_capacity);
void free_dictionary_compressor(DictionaryCompressor* compressor);

CompressBuffer* create_compress_buffer(size_t initial_capacity);
void resize_compress_buffer(CompressBuffer* buffer, size_t new_capacity);
void free_compress_buffer(CompressBuffer* buffer);

typedef struct {
    size_t original_size;
    size_t compressed_size;
    float compression_ratio;
    float entropy;
    size_t unique_bytes;
    uint32_t byte_frequencies[256];
} CompressionStats;

CompressionStats analyze_compression_potential(const uint8_t* data, size_t size);
float calculate_entropy(const uint32_t* frequencies, size_t total_count);

#ifdef __cplusplus
}
#endif

#endif
