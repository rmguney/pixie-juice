#include "video_encode.h"
#include "util.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Helper to create error result
static VideoEncodeResult create_encode_error_result(const char* error_msg) {
    VideoEncodeResult result = {0};
    result.success = 0;
    strncpy(result.error_message, error_msg, sizeof(result.error_message) - 1);
    result.error_message[sizeof(result.error_message) - 1] = '\0';
    return result;
}

// Helper to create success result
static VideoEncodeResult create_encode_success_result(uint8_t* data, size_t size,
                                                     int width, int height, double fps) {
    VideoEncodeResult result = {0};
    result.data = data;
    result.size = size;
    result.width = width;
    result.height = height;
    result.fps = fps;
    result.success = 1;
    return result;
}

VideoEncodeResult encode_h264(const uint8_t* frame_data, size_t frame_count,
                             const VideoEncodeParams* params) {
    if (!frame_data || !params || frame_count == 0) {
        return create_encode_error_result("Invalid input data");
    }
    
    if (params->width <= 0 || params->height <= 0 || params->fps <= 0) {
        return create_encode_error_result("Invalid video dimensions or fps");
    }
    
    if (params->crf < 0 || params->crf > 51) {
        return create_encode_error_result("CRF must be between 0 and 51");
    }
    
    // TODO: Integrate x264 library for high-performance H.264 encoding
    // This is a performance-critical hotspot that justifies C implementation
    // x264 provides much better performance than pure Rust alternatives
    
    // Placeholder: create dummy encoded data
    size_t estimated_size = (params->width * params->height * frame_count) / 10; // ~10:1 compression
    uint8_t* output_data = malloc(estimated_size);
    
    if (!output_data) {
        return create_encode_error_result("Memory allocation failed");
    }
    
    // Fill with placeholder H.264 header-like data
    memset(output_data, 0x00, estimated_size);
    if (estimated_size >= 4) {
        output_data[0] = 0x00;
        output_data[1] = 0x00;
        output_data[2] = 0x00;
        output_data[3] = 0x01; // H.264 NAL unit start code
    }
    
    return create_encode_success_result(output_data, estimated_size, 
                                       params->width, params->height, params->fps);
}

VideoEncodeResult encode_h265(const uint8_t* frame_data, size_t frame_count,
                             const VideoEncodeParams* params) {
    if (!frame_data || !params || frame_count == 0) {
        return create_encode_error_result("Invalid input data");
    }
    
    if (params->width <= 0 || params->height <= 0 || params->fps <= 0) {
        return create_encode_error_result("Invalid video dimensions or fps");
    }
    
    if (params->crf < 0 || params->crf > 51) {
        return create_encode_error_result("CRF must be between 0 and 51");
    }
    
    // TODO: Integrate x265 library for high-performance H.265 encoding
    // H.265 provides better compression but is more computationally intensive
    
    // Placeholder implementation
    size_t estimated_size = (params->width * params->height * frame_count) / 15; // ~15:1 compression
    uint8_t* output_data = malloc(estimated_size);
    
    if (!output_data) {
        return create_encode_error_result("Memory allocation failed");
    }
    
    memset(output_data, 0x00, estimated_size);
    if (estimated_size >= 4) {
        output_data[0] = 0x00;
        output_data[1] = 0x00;
        output_data[2] = 0x00;
        output_data[3] = 0x01; // H.265 NAL unit start code
    }
    
    return create_encode_success_result(output_data, estimated_size,
                                       params->width, params->height, params->fps);
}

VideoEncodeResult encode_webm(const uint8_t* frame_data, size_t frame_count,
                             const VideoEncodeParams* params) {
    if (!frame_data || !params || frame_count == 0) {
        return create_encode_error_result("Invalid input data");
    }
    
    if (params->width <= 0 || params->height <= 0 || params->fps <= 0) {
        return create_encode_error_result("Invalid video dimensions or fps");
    }
    
    // TODO: Integrate libvpx for VP9 encoding in WebM container
    // WebM is essential for web compatibility and WASM targets
    
    // Placeholder implementation
    size_t estimated_size = (params->width * params->height * frame_count) / 12; // ~12:1 compression
    uint8_t* output_data = malloc(estimated_size);
    
    if (!output_data) {
        return create_encode_error_result("Memory allocation failed");
    }
    
    memset(output_data, 0x00, estimated_size);
    if (estimated_size >= 4) {
        // WebM/Matroska EBML header signature
        output_data[0] = 0x1A;
        output_data[1] = 0x45;
        output_data[2] = 0xDF;
        output_data[3] = 0xA3;
    }
    
    return create_encode_success_result(output_data, estimated_size,
                                       params->width, params->height, params->fps);
}

void free_video_encode_result(VideoEncodeResult* result) {
    if (result && result->data) {
        free(result->data);
        result->data = NULL;
        result->size = 0;
        result->success = 0;
    }
}
