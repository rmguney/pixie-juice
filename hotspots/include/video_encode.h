#ifndef VIDEO_ENCODE_H
#define VIDEO_ENCODE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Video encoding result structure
typedef struct {
    uint8_t* data;
    size_t size;
    int width;
    int height;
    double fps;
    int success;
    char error_message[256];
} VideoEncodeResult;

// Video encoding parameters
typedef struct {
    int width;
    int height;
    double fps;
    int crf;              // Constant Rate Factor (0-51)
    int preset;           // Encoding preset (0=ultrafast, 9=placebo)
    int profile;          // H.264 profile (baseline=0, main=1, high=2)
    int keyframe_interval;
} VideoEncodeParams;

// High-performance H.264 encoding using x264 (performance hotspot)
VideoEncodeResult encode_h264(const uint8_t* frame_data, size_t frame_count,
                             const VideoEncodeParams* params);

// High-performance H.265 encoding using x265 (performance hotspot)
VideoEncodeResult encode_h265(const uint8_t* frame_data, size_t frame_count,
                             const VideoEncodeParams* params);

// WebM/VP9 encoding for web compatibility
VideoEncodeResult encode_webm(const uint8_t* frame_data, size_t frame_count,
                             const VideoEncodeParams* params);

// Memory cleanup
void free_video_encode_result(VideoEncodeResult* result);

#ifdef __cplusplus
}
#endif

#endif // VIDEO_ENCODE_H
