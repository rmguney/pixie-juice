#pragma once

// HEIC decoding through libheif and libde265: a library apart from the core,
// so builds without it stay free of those LGPL libraries. The web app loads it
// as a WebAssembly module of its own, only when a HEIC file needs it. What it
// decodes is converted with pixie::convert_pixels.

#include "pixie/pixie.hpp"

namespace pixie::heif {

struct Decoded {
  /// The primary image, cropped and upright (rotation and mirroring applied), straight alpha.
  Pixels pixels;
  /// ICC profile of the pixels' colour space; empty for sRGB.
  Bytes icc;
};

[[nodiscard]] Result<Decoded> decode(ByteView input);

}  // namespace pixie::heif
