# libheif compiles its own JavaScript API (heif_emscripten.h, for libheif-js)
# into any Emscripten build. The HEIC module has its own, one-function API, so
# that one is left out: it would only add size.

include(${CMAKE_CURRENT_LIST_DIR}/Patch.cmake)

pixie_patch(libheif/api/libheif/heif.cc
"#if defined(__EMSCRIPTEN__) && !defined(__EMSCRIPTEN_STANDALONE_WASM__)
#include \"api_structs.h\"
#include \"heif_emscripten.h\"
#endif"
"// libheif's JavaScript API (heif_emscripten.h) is left out of this build.")
