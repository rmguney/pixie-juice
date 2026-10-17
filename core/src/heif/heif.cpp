// HEIC through libheif (with libde265 for HEVC): the primary image as 8-bit
// RGBA, with libheif applying the crop, rotation and mirroring the file
// declares. Its colour space comes as the file's ICC profile, or one built with
// Little CMS from the colour primaries an nclx box names.

#include "pixie/heif.hpp"

#include "internal.hpp"

#include <lcms2.h>
#include <libheif/heif.h>

#include <algorithm>
#include <memory>
#include <mutex>

namespace pixie::heif {

namespace {

using detail::fail;

template <class T, void (*Free)(T*)>
struct Deleter {
  void operator()(T* p) const { Free(p); }
};

void release_handle(heif_image_handle* handle) { heif_image_handle_release(handle); }
void release_image(heif_image* image) { heif_image_release(image); }

using Context = std::unique_ptr<heif_context, Deleter<heif_context, heif_context_free>>;
using Handle = std::unique_ptr<heif_image_handle, Deleter<heif_image_handle, release_handle>>;
using Image = std::unique_ptr<heif_image, Deleter<heif_image, release_image>>;
using DecodingOptions = std::unique_ptr<heif_decoding_options, Deleter<heif_decoding_options, heif_decoding_options_free>>;
using Nclx = std::unique_ptr<heif_color_profile_nclx, Deleter<heif_color_profile_nclx, heif_nclx_color_profile_free>>;

Result<Decoded> failure(const heif_error& error) {
  return fail(std::string("HEIC decode failed: ") + (error.message ? error.message : "unknown error"));
}

/// An ICC profile for the primaries and white point an nclx box names, with
/// the sRGB tone curve (or a linear one): the RGB libheif converts to is in
/// those primaries. Empty for sRGB primaries, which need no profile.
Bytes profile_from_nclx(const heif_color_profile_nclx& nclx) {
  const bool srgb_primaries = nclx.color_primaries == heif_color_primaries_ITU_R_BT_709_5 ||
                              nclx.color_primaries == heif_color_primaries_unspecified;
  const bool linear = nclx.transfer_characteristics == heif_transfer_characteristic_linear;
  if (srgb_primaries && !linear) return {};
  const cmsCIExyY white{nclx.color_primary_white_x, nclx.color_primary_white_y, 1.0};
  const cmsCIExyYTRIPLE primaries{
      {nclx.color_primary_red_x, nclx.color_primary_red_y, 1.0},
      {nclx.color_primary_green_x, nclx.color_primary_green_y, 1.0},
      {nclx.color_primary_blue_x, nclx.color_primary_blue_y, 1.0},
  };
  const cmsFloat64Number srgb_curve[] = {2.4, 1 / 1.055, 0.055 / 1.055, 1 / 12.92, 0.04045};
  cmsToneCurve* curve = linear ? cmsBuildGamma(nullptr, 1.0) : cmsBuildParametricToneCurve(nullptr, 4, srgb_curve);
  if (!curve) return {};
  cmsToneCurve* curves[3] = {curve, curve, curve};
  cmsHPROFILE profile = cmsCreateRGBProfile(&white, &primaries, curves);
  cmsFreeToneCurve(curve);
  if (!profile) return {};
  Bytes icc;
  cmsUInt32Number size = 0;
  if (cmsSaveProfileToMem(profile, nullptr, &size) && size > 0) {
    icc.resize(size);
    if (!cmsSaveProfileToMem(profile, icc.data(), &size)) icc.clear();
  }
  cmsCloseProfile(profile);
  return icc;
}

Bytes colour_profile(const heif_image_handle* handle) {
  switch (heif_image_handle_get_color_profile_type(handle)) {
    case heif_color_profile_type_prof:
    case heif_color_profile_type_rICC: {
      Bytes icc(heif_image_handle_get_raw_color_profile_size(handle));
      if (icc.empty() || heif_image_handle_get_raw_color_profile(handle, icc.data()).code != heif_error_Ok) return {};
      return icc;
    }
    case heif_color_profile_type_nclx: {
      heif_color_profile_nclx* raw = nullptr;
      if (heif_image_handle_get_nclx_color_profile(handle, &raw).code != heif_error_Ok || !raw) return {};
      const Nclx nclx(raw);
      return profile_from_nclx(*nclx);
    }
    default: return {};
  }
}

}  // namespace

Result<Decoded> decode(ByteView input) {
  static std::once_flag initialized;
  std::call_once(initialized, [] { heif_init(nullptr); });

  const Context context(heif_context_alloc());
  if (!context) return fail("HEIC decode failed: out of memory");
  // The same image size cap as every other decoder here, and a ceiling on what libheif allocates.
  if (heif_security_limits* limits = heif_context_get_security_limits(context.get())) {
    limits->max_image_size_pixels = detail::kMaxPixels;
    if (limits->version >= 2) limits->max_total_memory = std::uint64_t(1) << 30;
  }
  if (const auto error = heif_context_read_from_memory_without_copy(context.get(), input.data(), input.size(), nullptr);
      error.code != heif_error_Ok) {
    return failure(error);
  }
  heif_image_handle* raw_handle = nullptr;
  if (const auto error = heif_context_get_primary_image_handle(context.get(), &raw_handle); error.code != heif_error_Ok) {
    return failure(error);
  }
  const Handle handle(raw_handle);

  const DecodingOptions options(heif_decoding_options_alloc());
  if (!options) return fail("HEIC decode failed: out of memory");
  options->ignore_transformations = 0;
  options->convert_hdr_to_8bit = 1;  // 10-bit HEIC (HDR iPhone photos) into 8-bit RGBA
  heif_image* raw_image = nullptr;
  if (const auto error = heif_decode_image(handle.get(), &raw_image, heif_colorspace_RGB, heif_chroma_interleaved_RGBA,
                                           options.get());
      error.code != heif_error_Ok) {
    return failure(error);
  }
  const Image image(raw_image);

  const int width = heif_image_get_width(image.get(), heif_channel_interleaved);
  const int height = heif_image_get_height(image.get(), heif_channel_interleaved);
  std::size_t stride = 0;
  const std::uint8_t* plane = heif_image_get_plane_readonly2(image.get(), heif_channel_interleaved, &stride);
  if (!plane || width <= 0 || height <= 0 || !detail::pixels_in_range(std::uint64_t(width), std::uint64_t(height)) ||
      stride < std::size_t(width) * 4) {
    return fail("HEIC decode failed: no usable image");
  }

  Decoded decoded;
  decoded.pixels.width = std::uint32_t(width);
  decoded.pixels.height = std::uint32_t(height);
  const std::size_t row = std::size_t(width) * 4;
  decoded.pixels.rgba.resize(row * std::size_t(height));
  for (std::size_t y = 0; y < std::size_t(height); ++y) {
    std::copy_n(plane + y * stride, row, decoded.pixels.rgba.data() + y * row);
  }
  // Alpha the file stores premultiplied comes out premultiplied: everything here takes straight alpha.
  if (heif_image_is_premultiplied_alpha(image.get())) detail::unpremultiply(decoded.pixels.rgba);
  decoded.icc = colour_profile(handle.get());
  return decoded;
}

}  // namespace pixie::heif
