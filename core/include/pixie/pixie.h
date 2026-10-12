#ifndef PIXIE_H
#define PIXIE_H

/*
 * Pixie Juice C API: the engine of pixie.hpp for C and for any language with
 * a C FFI (Python ctypes/cffi, C#, Go, ...). The "pixie" shared library
 * (pixie.dll, libpixie.so, libpixie.dylib) exports exactly these functions.
 *
 * Results own their memory until pixie_result_free / pixie_image_free; every
 * const char* that isn't an error is a static string. Calls keep no state
 * between them and may run on several threads at once.
 */

#include <stddef.h>
#include <stdint.h>

#if defined(PIXIE_BUILDING_SHARED) && defined(_WIN32)
#define PIXIE_API __declspec(dllexport)
#elif defined(PIXIE_BUILDING_SHARED) && defined(__GNUC__)
#define PIXIE_API __attribute__((visibility("default")))
#else
#define PIXIE_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct pixie_options {
  int quality;           /* 1-100: lossy encoder quality and the fidelity check's error budget */
  int lossless;          /* nonzero: never change pixels or geometry */
  int preserve_metadata; /* nonzero: keep EXIF, XMP, comments and other non-visual metadata */
  float mesh_ratio;      /* share of mesh triangles to keep, (0, 1] */
} pixie_options;

/* Library-owned storage behind a result. */
typedef struct pixie_storage pixie_storage;

typedef struct pixie_result {
  const uint8_t* data; /* the new file; NULL when unchanged (the input is the result) or on error */
  size_t size;
  const char* format;  /* output format name ("png", "glb", ...); NULL on error */
  int changed;         /* 0 when nothing smaller passed verification */
  const char* error;   /* NULL on success, otherwise why it failed */
  pixie_storage* storage;
} pixie_result;

typedef struct pixie_image {
  uint32_t width;
  uint32_t height;
  const uint8_t* rgba; /* width * height * 4 bytes, row-major; NULL on error */
  const char* error;   /* NULL on success, otherwise why it failed */
  pixie_storage* storage;
} pixie_image;

typedef struct pixie_mesh {
  const float* positions;  /* xyz per vertex; NULL on error */
  const float* normals;    /* xyz per vertex, or NULL */
  const float* colors;     /* rgb in 0..1 per vertex, or NULL */
  size_t vertex_count;
  const uint32_t* indices; /* three per triangle */
  size_t index_count;
  const char* error;       /* NULL on success, otherwise why it failed */
  pixie_storage* storage;
} pixie_mesh;

/* Quality 75, lossy, metadata stripped, every triangle kept. */
PIXIE_API pixie_options pixie_default_options(void);

/* Format name of the data ("png", "jpeg", "obj", ...) or "unknown". */
PIXIE_API const char* pixie_detect(const uint8_t* data, size_t size);

/* Optimizes within the input's own format; see pixie::optimize. NULL options means the defaults. */
PIXIE_API pixie_result pixie_optimize(const uint8_t* data, size_t size, const pixie_options* options);

/* Converts a raster image to "png", "jpeg", "webp", "gif", "bmp", "tga", "ico" or "tiff";
   see pixie::convert. NULL options means the defaults. */
PIXIE_API pixie_result pixie_convert(const uint8_t* data, size_t size, const char* target,
                                     const pixie_options* options);

/* Decodes the first frame of a raster image as upright sRGB RGBA; see pixie::decode. */
PIXIE_API pixie_image pixie_decode(const uint8_t* data, size_t size);

/* Reads the triangles of an OBJ, STL or PLY for display; see pixie::decode_mesh. */
PIXIE_API pixie_mesh pixie_decode_mesh(const uint8_t* data, size_t size);

/* Release a result, image or mesh and reset it to zero; safe to call twice. */
PIXIE_API void pixie_result_free(pixie_result* result);
PIXIE_API void pixie_image_free(pixie_image* image);
PIXIE_API void pixie_mesh_free(pixie_mesh* mesh);

/* Engine version, e.g. "0.2.0". */
PIXIE_API const char* pixie_version(void);

#ifdef __cplusplus
}
#endif

#endif /* PIXIE_H */
