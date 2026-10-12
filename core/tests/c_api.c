/* Compiled as C, so the header is checked as C and the API is exercised from C. */

#include "pixie/pixie.h"

#include <string.h>

/* 0 when every check passes, otherwise the number of the first failed check. */
int pixie_c_api_check(const uint8_t* png, size_t png_size, uint32_t width, uint32_t height) {
  if (strcmp(pixie_detect(png, png_size), "png") != 0) return 1;

  pixie_options options = pixie_default_options();
  if (options.quality != 75 || options.lossless || options.mesh_ratio != 1.0f) return 2;
  options.lossless = 1;
  pixie_result optimized = pixie_optimize(png, png_size, &options);
  if (optimized.error || strcmp(optimized.format, "png") != 0) return 3;
  if (optimized.changed ? !optimized.data || optimized.size >= png_size : optimized.data != NULL) return 4;
  pixie_result_free(&optimized);
  pixie_result_free(&optimized); /* safe twice */

  pixie_result converted = pixie_convert(png, png_size, "webp", NULL);
  if (converted.error || !converted.changed || strcmp(converted.format, "webp") != 0) return 5;
  pixie_image image = pixie_decode(converted.data, converted.size);
  if (image.error || image.width != width || image.height != height || !image.rgba) return 6;
  pixie_image_free(&image);
  pixie_result_free(&converted);

  pixie_result refused = pixie_convert(png, png_size, "svg", NULL);
  if (!refused.error || refused.data || refused.format) return 7;
  pixie_result_free(&refused);

  static const uint8_t garbage[] = {1, 2, 3, 4, 5, 6, 7, 8};
  pixie_result failed = pixie_optimize(garbage, sizeof garbage, NULL);
  if (!failed.error) return 8;
  pixie_result_free(&failed);
  pixie_image undecodable = pixie_decode(NULL, 0);
  if (!undecodable.error || undecodable.rgba) return 9;
  pixie_image_free(&undecodable);

  static const char triangle[] = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
  pixie_mesh mesh = pixie_decode_mesh((const uint8_t*)triangle, sizeof triangle - 1);
  if (mesh.error || mesh.vertex_count != 3 || mesh.index_count != 3 || mesh.normals || mesh.positions[3] != 1.0f) {
    return 10;
  }
  pixie_mesh_free(&mesh);
  pixie_mesh no_mesh = pixie_decode_mesh(png, png_size);
  if (!no_mesh.error || no_mesh.positions) return 11;
  pixie_mesh_free(&no_mesh);

  return strcmp(pixie_version(), "") != 0 ? 0 : 12;
}
