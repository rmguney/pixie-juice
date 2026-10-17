// WebP through libwebp.
//
// Candidates:
//  1. EXIF and XMP chunks removed with libwebpmux (pixels, animation and the
//     ICC profile untouched), unless metadata is preserved; a non-default
//     orientation is kept as a minimal EXIF chunk;
//  2. for still images: a lossless re-encode of lossless sources (exact RGB
//     under transparent pixels), and a lossy re-encode at the requested quality
//     unless lossless, with the same metadata policy and the ICC profile.

#include "internal.hpp"

#include <webp/decode.h>
#include <webp/encode.h>
#include <webp/mux.h>


namespace pixie::detail {

namespace {

constexpr std::size_t kLargeImage = 1024 * 1024;

std::optional<Bytes> assemble(WebPMux* mux) {
  WebPData out;
  WebPDataInit(&out);
  if (WebPMuxAssemble(mux, &out) != WEBP_MUX_OK) return std::nullopt;
  Bytes bytes(out.bytes, out.bytes + out.size);
  WebPDataClear(&out);
  return bytes;
}

/// Applies the metadata policy to `target`: EXIF and XMP go unless metadata is
/// preserved, and a non-default orientation survives as a minimal EXIF chunk.
/// When `source` is another file, its ICC profile (and preserved metadata) is copied in.
void apply_metadata(WebPMux* target, WebPMux* source, bool preserve_metadata, int orientation) {
  WebPData chunk;
  if (target != source) {
    for (const char* id : {"ICCP", "EXIF", "XMP "}) {
      if ((preserve_metadata || std::string_view(id) == "ICCP") && WebPMuxGetChunk(source, id, &chunk) == WEBP_MUX_OK) {
        WebPMuxSetChunk(target, id, &chunk, 1);
      }
    }
  }
  if (preserve_metadata) return;
  WebPMuxDeleteChunk(target, "XMP ");
  WebPMuxDeleteChunk(target, "EXIF");
  if (orientation != 1) {
    const Bytes exif = minimal_exif(orientation);
    const WebPData data{exif.data(), exif.size()};
    WebPMuxSetChunk(target, "EXIF", &data, 1);
  }
}

}  // namespace

Result<Bytes> encode_webp(const Image& image, int quality, bool lossless, ByteView icc) {
  WebPConfig config;
  if (!WebPConfigInit(&config)) return fail("WebP encoder init failed");
  const bool large = image.pixel_count() > kLargeImage;
  if (lossless) {
    WebPConfigLosslessPreset(&config, large ? 6 : 9);
    config.exact = 1;  // keep RGB under transparent pixels
  } else {
    config.quality = float(quality);
    config.method = large ? 4 : 6;
    config.alpha_quality = 100;
    config.use_sharp_yuv = 1;
  }

  WebPPicture picture;
  if (!WebPPictureInit(&picture)) return fail("WebP picture init failed");
  picture.use_argb = 1;
  picture.width = int(image.width);
  picture.height = int(image.height);
  if (!WebPPictureImportRGBA(&picture, image.rgba.data(), int(image.width) * 4)) {
    WebPPictureFree(&picture);
    return fail("WebP import failed");
  }
  WebPMemoryWriter writer;
  WebPMemoryWriterInit(&writer);
  picture.writer = WebPMemoryWrite;
  picture.custom_ptr = &writer;
  const int ok = WebPEncode(&config, &picture);
  WebPPictureFree(&picture);
  if (!ok) {
    WebPMemoryWriterClear(&writer);
    return fail("WebP encode failed");
  }
  Bytes out(writer.mem, writer.mem + writer.size);
  WebPMemoryWriterClear(&writer);
  if (icc.empty()) return out;

  const WebPData data{out.data(), out.size()};
  const WebPData profile{icc.data(), icc.size()};
  WebPMux* mux = WebPMuxCreate(&data, 0);
  std::optional<Bytes> with_icc;
  if (mux && WebPMuxSetChunk(mux, "ICCP", &profile, 1) == WEBP_MUX_OK) with_icc = assemble(mux);
  WebPMuxDelete(mux);
  if (!with_icc) return fail("Could not embed the ICC profile");
  return std::move(*with_icc);
}

std::vector<Bytes> webp_candidates(ByteView input, const Image& image, const Options& options) {
  std::vector<Bytes> candidates;
  const WebPData data{input.data(), input.size()};
  WebPMux* source = WebPMuxCreate(&data, 0);
  if (!source) return candidates;
  const int orientation = read_colour_info(input, Format::WebP).orientation;

  if (!options.preserve_metadata) {
    WebPMux* stripped = WebPMuxCreate(&data, 0);
    if (stripped) {
      apply_metadata(stripped, stripped, false, orientation);
      if (auto out = assemble(stripped)) candidates.push_back(std::move(*out));
      WebPMuxDelete(stripped);
    }
  }

  WebPBitstreamFeatures features;
  // Animated WebP: metadata only for now.
  if (WebPGetFeatures(input.data(), input.size(), &features) == VP8_STATUS_OK && !features.has_animation) {
    auto add = [&](Result<Bytes> encoded) {
      if (!encoded) return;
      const WebPData encoded_data{encoded->data(), encoded->size()};
      WebPMux* target = WebPMuxCreate(&encoded_data, 0);
      if (!target) return;
      apply_metadata(target, source, options.preserve_metadata, orientation);
      if (auto out = assemble(target)) candidates.push_back(std::move(*out));
      WebPMuxDelete(target);
    };
    const bool source_lossless = features.format == 2;
    if (source_lossless) add(encode_webp(image, 100, true));
    if (!options.lossless) add(encode_webp(image, options.quality, false));
  }
  WebPMuxDelete(source);
  return candidates;
}

bool webp_is_animated(ByteView data) {
  WebPBitstreamFeatures features;
  return WebPGetFeatures(data.data(), data.size(), &features) == VP8_STATUS_OK && features.has_animation;
}

}  // namespace pixie::detail
