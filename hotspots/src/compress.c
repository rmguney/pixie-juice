#include "compress.h"

extern void* wasm_malloc(size_t size);
extern void wasm_free(void* ptr);

static inline uint32_t hash_u32(uint32_t x) {
    x ^= x >> 16;
    x *= 0x85ebca6b;
    x ^= x >> 13;
    x *= 0xc2b2ae35;
    x ^= x >> 16;
    return x;
}

static inline uint32_t rotl32(uint32_t x, int8_t r) {
    return (x << r) | (x >> (32 - r));
}

#define LZ4_MINMATCH 4
#define LZ4_COPYLENGTH 8
#define LZ4_LASTLITERALS 5
#define LZ4_MFLIMIT (LZ4_COPYLENGTH + LZ4_MINMATCH)
#define LZ4_ACCELERATION_DEFAULT 1
#define LZ4_HASH_SIZE_U32 (1 << 12)
#define LZ4_DISTANCE_MAX 65535

typedef struct {
    const uint8_t* base;
    const uint8_t* low_limit;
    const uint8_t* next_to_update;
    uint32_t table[LZ4_HASH_SIZE_U32];
} LZ4_stream_t;

// LZ4 hash function
static inline uint32_t lz4_hash_sequence(uint32_t sequence, uint32_t table_type) {
    if (table_type == 0) {
        return ((sequence * 2654435761U) >> 20);
    } else {
        return ((sequence * 2654435761U) >> 12);
    }
}

// LZ4 compression core
static size_t lz4_compress_generic(LZ4_stream_t* const ctx,
                                   const char* const src,
                                   char* const dst,
                                   const size_t src_size,
                                   const size_t dst_capacity,
                                   const uint32_t acceleration) {
    const uint8_t* ip = (const uint8_t*)src;
    const uint8_t* base = (const uint8_t*)src;
    const uint8_t* const low_prefix_ptr = base;
    const uint8_t* const dict_end = (const uint8_t*)src;
    const uint8_t* const source = (const uint8_t*)src;
    const uint8_t* const source_end = ip + src_size;
    const uint8_t* const mf_limit = source_end - LZ4_MFLIMIT;
    
    uint8_t* op = (uint8_t*)dst;
    uint8_t* const op_limit = op + dst_capacity;
    
    uint32_t forward_h;
    size_t ref_delta = 0;
    const uint8_t* anchor = ip;
    
    if (src_size < LZ4_MINMATCH + 1) goto _last_literals;
    
    ctx->table[lz4_hash_sequence(*(uint32_t*)ip, 0)] = (uint32_t)(ip - base);
    ip++;
    forward_h = lz4_hash_sequence(*(uint32_t*)ip, 0);
    
    for (;;) {
        const uint8_t* match;
        uint8_t* token;
        anchor = ip;
        size_t find_match_attempts = acceleration;
        
_next_match:
        {
            uint32_t const h = forward_h;
            uint32_t step = 1;
            uint32_t search_match_nb = find_match_attempts;
            
            do {
                uint32_t const match_index = ctx->table[h];
                ctx->table[h] = (uint32_t)(ip - base);
                
                if (match_index < (uint32_t)(ip - base) && 
                    (ip - base) - match_index <= LZ4_DISTANCE_MAX) {
                    
                    match = base + match_index;
                    ref_delta = (size_t)(ip - match);
                    break;
                }
                
                ip += step;
                if (ip > mf_limit) goto _last_literals;
                step = search_match_nb++ >> acceleration;
                forward_h = lz4_hash_sequence(*(uint32_t*)ip, 0);
            } while (1);
        }
        
        if (*(uint32_t*)match != *(uint32_t*)ip) {
            goto _next_match;
        }
        
        while ((ip > anchor) && (match > low_prefix_ptr) && (ip[-1] == match[-1])) {
            ip--;
            match--;
        }
        
        {
            uint32_t const literal_l = (uint32_t)(ip - anchor);
            token = op++;
            if ((op + literal_l + (2 + 1 + 5)) >= op_limit) return 0;
            
            if (literal_l >= 15) {
                int len = (int)literal_l - 15;
                *token = (15 << 4);
                while (len >= 255) {
                    *op++ = 255;
                    len -= 255;
                }
                *op++ = (uint8_t)len;
            } else {
                *token = (uint8_t)(literal_l << 4);
            }
            
            const uint8_t* end = op + literal_l;
            while (op < end) *op++ = *anchor++;
        }
        
_next_sequence:
        if (op + 2 >= op_limit) return 0;
        *op++ = (uint8_t)ref_delta;
        *op++ = (uint8_t)(ref_delta >> 8);
        
        ip += LZ4_MINMATCH;
        match += LZ4_MINMATCH;
        
        {
            const uint8_t* const match_end = match + (source_end - ip);
            const uint8_t* const match_limit = match_end < (ip + 65535) ? 
                                               match_end : ip + 65535;
            
            while (ip < match_limit && *ip == *match) {
                ip++;
                match++;
            }
        }
        
        {
            uint32_t const ml = (uint32_t)(ip - (anchor + LZ4_MINMATCH));
            if (op + (1 + (ml >= 15 ? (ml - 15) / 255 + 1 : 0)) > op_limit) return 0;
            
            if (ml >= 15) {
                int len = (int)ml - 15;
                *token += 15;
                while (len >= 255) {
                    *op++ = 255;
                    len -= 255;
                }
                *op++ = (uint8_t)len;
            } else {
                *token += (uint8_t)ml;
            }
        }
        
        if (ip >= mf_limit) break;
        ctx->table[lz4_hash_sequence(*(uint32_t*)(ip - 2), 0)] = (uint32_t)(ip - 2 - base);
        forward_h = lz4_hash_sequence(*(uint32_t*)ip, 0);
    }
    
_last_literals:
    {
        size_t const last_run = (size_t)(source_end - anchor);
        if (op + last_run + 1 + ((last_run + 255 - 15) / 255) >= op_limit) return 0;
        
        if (last_run >= 15) {
            size_t accumulator = last_run - 15;
            *op++ = 15 << 4;
            while (accumulator >= 255) {
                *op++ = 255;
                accumulator -= 255;
            }
            *op++ = (uint8_t)accumulator;
        } else {
            *op++ = (uint8_t)(last_run << 4);
        }
        
        const uint8_t* end = op + last_run;
        while (op < end) *op++ = *anchor++;
    }
    
    return (size_t)(op - (uint8_t*)dst);
}

// LZ4 decompression
static size_t lz4_decompress_safe(const char* src, char* dst, 
                                  size_t src_size, size_t dst_capacity) {
    const uint8_t* ip = (const uint8_t*)src;
    const uint8_t* const ip_end = ip + src_size;
    
    uint8_t* op = (uint8_t*)dst;
    uint8_t* const op_end = op + dst_capacity;
    
    while (ip < ip_end) {
        uint32_t token = *ip++;
        uint32_t literal_length = token >> 4;
        
        if (literal_length == 15) {
            uint32_t s;
            do {
                if (ip >= ip_end) return 0;
                s = *ip++;
                literal_length += s;
            } while (s == 255);
        }
        
        if (op + literal_length > op_end) return 0;
        if (ip + literal_length > ip_end) return 0;
        
        for (uint32_t i = 0; i < literal_length; i++) {
            *op++ = *ip++;
        }
        
        if (ip >= ip_end) break;
        
        if (ip + 1 >= ip_end) return 0;
        uint32_t offset = *ip++;
        offset |= (*ip++) << 8;
        
        if (offset == 0) return 0;
        
        uint32_t match_length = token & 15;
        if (match_length == 15) {
            uint32_t s;
            do {
                if (ip >= ip_end) return 0;
                s = *ip++;
                match_length += s;
            } while (s == 255);
        }
        match_length += LZ4_MINMATCH;
        
        if (op + match_length > op_end) return 0;
        if (op - offset < (uint8_t*)dst) return 0;
        
        const uint8_t* match = op - offset;
        for (uint32_t i = 0; i < match_length; i++) {
            *op++ = *match++;
        }
    }
    
    return (size_t)(op - (uint8_t*)dst);
}

#define HUFFMAN_MAX_SYMBOLS 256
#define HUFFMAN_MAX_CODE_LENGTH 15

typedef struct {
    uint16_t symbol;
    uint16_t code_length;
    uint32_t code;
} HuffmanCode;

typedef struct {
    uint32_t frequencies[HUFFMAN_MAX_SYMBOLS];
    HuffmanCode codes[HUFFMAN_MAX_SYMBOLS];
    uint32_t symbol_count;
} HuffmanEncoder;

typedef struct HuffmanNode {
    uint32_t frequency;
    uint16_t symbol;
    uint8_t is_leaf;
    struct HuffmanNode* left;
    struct HuffmanNode* right;
} HuffmanNode;

typedef struct {
    HuffmanNode** nodes;
    size_t count;
    size_t capacity;
} PriorityQueue;

static void pq_init(PriorityQueue* pq, size_t capacity) {
    pq->nodes = (HuffmanNode**)wasm_malloc(capacity * sizeof(HuffmanNode*));
    pq->count = 0;
    pq->capacity = capacity;
}

static void pq_free(PriorityQueue* pq) {
    wasm_free(pq->nodes);
    pq->nodes = NULL;
    pq->count = 0;
}

static void pq_heapify_up(PriorityQueue* pq, size_t index) {
    while (index > 0) {
        size_t parent = (index - 1) / 2;
        if (pq->nodes[index]->frequency >= pq->nodes[parent]->frequency) break;
        
        HuffmanNode* temp = pq->nodes[index];
        pq->nodes[index] = pq->nodes[parent];
        pq->nodes[parent] = temp;
        
        index = parent;
    }
}

static void pq_heapify_down(PriorityQueue* pq, size_t index) {
    while (index * 2 + 1 < pq->count) {
        size_t left = index * 2 + 1;
        size_t right = index * 2 + 2;
        size_t smallest = index;
        
        if (pq->nodes[left]->frequency < pq->nodes[smallest]->frequency) {
            smallest = left;
        }
        
        if (right < pq->count && pq->nodes[right]->frequency < pq->nodes[smallest]->frequency) {
            smallest = right;
        }
        
        if (smallest == index) break;
        
        HuffmanNode* temp = pq->nodes[index];
        pq->nodes[index] = pq->nodes[smallest];
        pq->nodes[smallest] = temp;
        
        index = smallest;
    }
}

static void pq_push(PriorityQueue* pq, HuffmanNode* node) {
    if (pq->count >= pq->capacity) return;
    
    pq->nodes[pq->count] = node;
    pq_heapify_up(pq, pq->count);
    pq->count++;
}

static HuffmanNode* pq_pop(PriorityQueue* pq) {
    if (pq->count == 0) return NULL;
    
    HuffmanNode* result = pq->nodes[0];
    pq->nodes[0] = pq->nodes[pq->count - 1];
    pq->count--;
    
    if (pq->count > 0) {
        pq_heapify_down(pq, 0);
    }
    
    return result;
}

static void generate_codes_recursive(HuffmanNode* node, uint32_t code, 
                                     uint8_t depth, HuffmanEncoder* encoder) {
    if (!node) return;
    
    if (node->is_leaf) {
        encoder->codes[node->symbol].symbol = node->symbol;
        encoder->codes[node->symbol].code = code;
        encoder->codes[node->symbol].code_length = depth;
        return;
    }
    
    generate_codes_recursive(node->left, code << 1, depth + 1, encoder);
    generate_codes_recursive(node->right, (code << 1) | 1, depth + 1, encoder);
}

WASM_EXPORT int32_t compress_lz4(const uint8_t* input, size_t input_size, 
                     uint8_t* output, size_t max_output_size) {
    if (!input || !output || input_size == 0 || max_output_size < 16) {
        return -1;
    }
    
    LZ4_stream_t ctx;
    for (int i = 0; i < LZ4_HASH_SIZE_U32; i++) {
        ctx.table[i] = 0;
    }
    ctx.base = input;
    
    size_t compressed_size = lz4_compress_generic(&ctx, (const char*)input, 
                                                  (char*)output, input_size, 
                                                  max_output_size, LZ4_ACCELERATION_DEFAULT);
    
    return compressed_size > 0 ? (int32_t)compressed_size : -1;
}

WASM_EXPORT int32_t decompress_lz4(const uint8_t* input, size_t input_size,
                       uint8_t* output, size_t max_output_size) {
    if (!input || !output || input_size == 0 || max_output_size == 0) {
        return -1;
    }
    
    size_t decompressed_size = lz4_decompress_safe((const char*)input, (char*)output,
                                                   input_size, max_output_size);
    
    return decompressed_size > 0 ? (int32_t)decompressed_size : -1;
}

WASM_EXPORT int32_t compress_huffman(const uint8_t* input, size_t input_size,
                         uint8_t* output, size_t max_output_size) {
    if (!input || !output || input_size == 0 || max_output_size < 1024) {
        return -1;
    }
    
    HuffmanEncoder encoder = {0};
    
    for (size_t i = 0; i < input_size; i++) {
        encoder.frequencies[input[i]]++;
    }
    
    encoder.symbol_count = 0;
    for (int i = 0; i < HUFFMAN_MAX_SYMBOLS; i++) {
        if (encoder.frequencies[i] > 0) {
            encoder.symbol_count++;
        }
    }
    
    if (encoder.symbol_count <= 1) {
        if (encoder.symbol_count == 1) {
            output[0] = 1;
            for (int i = 0; i < HUFFMAN_MAX_SYMBOLS; i++) {
                if (encoder.frequencies[i] > 0) {
                    output[1] = (uint8_t)i;
                    break;
                }
            }
            return 2;
        }
        return -1;
    }
    
    HuffmanNode* nodes = (HuffmanNode*)wasm_malloc(encoder.symbol_count * 2 * sizeof(HuffmanNode));
    if (!nodes) return -1;
    
    PriorityQueue pq;
    pq_init(&pq, encoder.symbol_count * 2);
    
    size_t node_index = 0;
    for (int i = 0; i < HUFFMAN_MAX_SYMBOLS; i++) {
        if (encoder.frequencies[i] > 0) {
            nodes[node_index].frequency = encoder.frequencies[i];
            nodes[node_index].symbol = (uint16_t)i;
            nodes[node_index].is_leaf = 1;
            nodes[node_index].left = NULL;
            nodes[node_index].right = NULL;
            pq_push(&pq, &nodes[node_index]);
            node_index++;
        }
    }
    
    while (pq.count > 1) {
        HuffmanNode* left = pq_pop(&pq);
        HuffmanNode* right = pq_pop(&pq);
        
        nodes[node_index].frequency = left->frequency + right->frequency;
        nodes[node_index].symbol = 0;
        nodes[node_index].is_leaf = 0;
        nodes[node_index].left = left;
        nodes[node_index].right = right;
        
        pq_push(&pq, &nodes[node_index]);
        node_index++;
    }
    
    HuffmanNode* root = pq_pop(&pq);
    
    generate_codes_recursive(root, 0, 0, &encoder);
    
    uint8_t* write_ptr = output;
    *write_ptr++ = 0;
    *write_ptr++ = (uint8_t)encoder.symbol_count;
    
    for (int i = 0; i < HUFFMAN_MAX_SYMBOLS; i++) {
        if (encoder.frequencies[i] > 0) {
            *write_ptr++ = (uint8_t)i;
            *write_ptr++ = encoder.codes[i].code_length;
            *write_ptr++ = (uint8_t)encoder.codes[i].code;
            *write_ptr++ = (uint8_t)(encoder.codes[i].code >> 8);
        }
    }
    
    uint32_t bit_buffer = 0;
    uint8_t bit_count = 0;
    
    for (size_t i = 0; i < input_size; i++) {
        uint8_t symbol = input[i];
        uint32_t code = encoder.codes[symbol].code;
        uint8_t code_length = encoder.codes[symbol].code_length;
        
        bit_buffer |= (code << bit_count);
        bit_count += code_length;
        
        while (bit_count >= 8) {
            if (write_ptr >= output + max_output_size) {
                wasm_free(nodes);
                pq_free(&pq);
                return -1;
            }
            *write_ptr++ = (uint8_t)bit_buffer;
            bit_buffer >>= 8;
            bit_count -= 8;
        }
    }
    
    if (bit_count > 0) {
        if (write_ptr >= output + max_output_size) {
            wasm_free(nodes);
            pq_free(&pq);
            return -1;
        }
        *write_ptr++ = (uint8_t)bit_buffer;
    }
    
    wasm_free(nodes);
    pq_free(&pq);
    
    return (int32_t)(write_ptr - output);
}

WASM_EXPORT CompressionMethod get_optimal_compression(const uint8_t* data, size_t size) {
    if (!data || size == 0) return METHOD_NONE;
    
    uint32_t unique_bytes = 0;
    uint32_t byte_counts[256] = {0};
    
    for (size_t i = 0; i < size && i < 4096; i++) {
        byte_counts[data[i]]++;
    }
    
    for (int i = 0; i < 256; i++) {
        if (byte_counts[i] > 0) unique_bytes++;
    }
    
    if (unique_bytes <= 16) {
        return METHOD_HUFFMAN;
    } else if (size > 1024) {
        return METHOD_LZ4; // Large data - LZ4 for speed
    } else {
        return METHOD_HUFFMAN;
    }
}
