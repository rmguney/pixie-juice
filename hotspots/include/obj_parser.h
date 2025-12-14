#ifndef OBJ_PARSER_H
#define OBJ_PARSER_H

#include "memory.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    float* vertices;
    size_t vertex_count;

    float* normals;
    size_t normal_count;

    float* texcoords;
    size_t texcoord_count;

    uint32_t* indices;
    size_t index_count;

    char* object_name;
    size_t object_name_len;

    int success;
    char error_message[256];
} ObjParseResult;

WASM_EXPORT ObjParseResult* obj_parse_to_mesh(const uint8_t* data, size_t data_len);
WASM_EXPORT void free_obj_parse_result(ObjParseResult* result);

#ifdef __cplusplus
}
#endif

#endif
