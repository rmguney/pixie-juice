# Third-party libraries at their latest release (or latest commit, for stb,
# miniply and lodepng, which have no releases), downloaded from each project's
# official repository and checked against a SHA-256: GitHub for all but two,
# giflib (SourceForge) and libtiff (GitLab, releases on download.osgeo.org).
# Every codec and mesh algorithm in Pixie Juice comes from one of these; src/
# only orchestrates them. A few carry small fixes (cmake/patches/), applied
# after download.

include(FetchContent)
set(FETCHCONTENT_QUIET ON)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)

set(PIXIE_PATCHES ${CMAKE_CURRENT_LIST_DIR}/patches)

# The arguments that patch a library with patches/<name>.cmake. The download is
# named after the scripts' hash: a step that changes only re-runs itself, so an
# edited patch would land on sources patched before; a new name fetches and
# extracts them afresh first.
function(pixie_patch out name)
  file(SHA256 ${PIXIE_PATCHES}/${name}.cmake script)
  file(SHA256 ${PIXIE_PATCHES}/Patch.cmake helpers)
  string(SHA256 hash "${script}${helpers}")
  string(SUBSTRING ${hash} 0 16 hash)
  set(${out}
    PATCH_COMMAND ${CMAKE_COMMAND} -P ${PIXIE_PATCHES}/${name}.cmake
    DOWNLOAD_NAME ${name}-${hash}.tar.gz
    PARENT_SCOPE)
endfunction()
foreach(name mozjpeg zopfli stb meshoptimizer miniply libtiff libde265 libheif)
  string(TOUPPER ${name} upper)
  pixie_patch(${upper}_PATCH ${name})
endforeach()

# pixie_fetch(<name> <url> <sha256> [FetchContent_Declare options...])
macro(pixie_fetch name url sha256)
  FetchContent_Declare(${name}
    URL ${url}
    URL_HASH SHA256=${sha256}
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
    ${ARGN})
  FetchContent_MakeAvailable(${name})
endmacro()

# --- JPEG: mozjpeg (libjpeg-turbo fork with trellis quantisation, progressive scans).
# libjpeg-turbo only supports being built as its own project, so it goes through
# ExternalProject with this build's toolchain and flags.
include(ExternalProject)
set(MOZJPEG_SOURCE ${CMAKE_BINARY_DIR}/_deps/mozjpeg-src)
set(MOZJPEG_BINARY ${CMAKE_BINARY_DIR}/_deps/mozjpeg-build)
if(MSVC)
  set(MOZJPEG_LIB ${MOZJPEG_BINARY}/jpeg-static.lib)
else()
  set(MOZJPEG_LIB ${MOZJPEG_BINARY}/${CMAKE_STATIC_LIBRARY_PREFIX}jpeg${CMAKE_STATIC_LIBRARY_SUFFIX})
endif()
# Sanitizer builds instrument mozjpeg too; as its own project it doesn't see this one's options.
set(MOZJPEG_C_FLAGS "${CMAKE_C_FLAGS}")
set(MOZJPEG_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS}")
if(PIXIE_FUZZ)
  string(APPEND MOZJPEG_C_FLAGS " -fsanitize=fuzzer-no-link")
endif()
if(PIXIE_SANITIZE)
  if(MSVC)
    string(APPEND MOZJPEG_C_FLAGS " /fsanitize=address")
  else()
    string(APPEND MOZJPEG_C_FLAGS " -fsanitize=address,undefined -fno-sanitize-recover=undefined -fno-omit-frame-pointer")
    string(APPEND MOZJPEG_LINKER_FLAGS " -fsanitize=address,undefined")
  endif()
endif()
set(MOZJPEG_ARGS
  -DCMAKE_BUILD_TYPE=Release
  "-DCMAKE_ARCHIVE_OUTPUT_DIRECTORY_RELEASE=${MOZJPEG_BINARY}"  # multi-config generators too
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5
  -DCMAKE_POSITION_INDEPENDENT_CODE=ON
  "-DCMAKE_C_FLAGS=${MOZJPEG_C_FLAGS}"
  "-DCMAKE_EXE_LINKER_FLAGS=${MOZJPEG_LINKER_FLAGS}"
  -DENABLE_SHARED=OFF
  -DENABLE_STATIC=ON
  -DWITH_SIMD=OFF            # NASM/x86 SIMD only; not available in WASM
  -DWITH_TURBOJPEG=OFF
  -DWITH_JAVA=OFF
  -DPNG_SUPPORTED=OFF
  -DWITH_ARITH_ENC=ON
  -DWITH_ARITH_DEC=ON
  -DWITH_CRT_DLL=ON)         # match the default /MD runtime on MSVC
if(CMAKE_TOOLCHAIN_FILE)
  list(APPEND MOZJPEG_ARGS "-DCMAKE_TOOLCHAIN_FILE=${CMAKE_TOOLCHAIN_FILE}")
else()  # this build's compiler, which the flags above are written for (a preset's clang isn't the default cc)
  list(APPEND MOZJPEG_ARGS "-DCMAKE_C_COMPILER=${CMAKE_C_COMPILER}")
endif()
if(CMAKE_MSVC_RUNTIME_LIBRARY)  # a C runtime chosen for this build applies to mozjpeg too
  list(APPEND MOZJPEG_ARGS -DCMAKE_POLICY_DEFAULT_CMP0091=NEW "-DCMAKE_MSVC_RUNTIME_LIBRARY=${CMAKE_MSVC_RUNTIME_LIBRARY}")
endif()
ExternalProject_Add(mozjpeg_ep
  URL https://github.com/mozilla/mozjpeg/archive/refs/tags/v4.1.5.tar.gz
  URL_HASH SHA256=9fcbb7171f6ac383f5b391175d6fb3acde5e64c4c4727274eade84ed0998fcc1
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE
  ${MOZJPEG_PATCH}
  SOURCE_DIR ${MOZJPEG_SOURCE}
  BINARY_DIR ${MOZJPEG_BINARY}
  CMAKE_ARGS ${MOZJPEG_ARGS}
  BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --target jpeg-static --config Release
  INSTALL_COMMAND ""
  BUILD_BYPRODUCTS ${MOZJPEG_LIB})
# jpeglib.h/jmorecfg.h/jerror.h live in the source tree, jconfig.h in the build tree.
file(MAKE_DIRECTORY ${MOZJPEG_SOURCE} ${MOZJPEG_BINARY})
add_library(pixie_jpeg STATIC IMPORTED GLOBAL)
set_target_properties(pixie_jpeg PROPERTIES
  IMPORTED_LOCATION ${MOZJPEG_LIB}
  INTERFACE_INCLUDE_DIRECTORIES "${MOZJPEG_SOURCE};${MOZJPEG_BINARY}")
add_dependencies(pixie_jpeg mozjpeg_ep)

# --- PNG: zopflipng (lossless PNG optimiser built on lodepng + zopfli deflate).
# lodepng comes from its own repository: zopfli 1.0.3 bundles lodepng 20191107,
# whose bit reader decodes some valid streams wrongly (large PNGs saved by
# current browsers fail their checksum). zopfli's copy is removed (see the patch).
pixie_fetch(lodepng
  https://github.com/lvandeve/lodepng/archive/e584a8490db1b76b81bf298c11c09d3b55d9c270.tar.gz
  c9cc1dcd143bd1365f0ce50cfc14bacab43d7bee5a2be212d6f4b73eefa8626f
  SOURCE_DIR ${FETCHCONTENT_BASE_DIR}/lodepng-src/lodepng  # zopflipng includes "lodepng/lodepng.h"
  SOURCE_SUBDIR _pixie_no_cmake)  # populate only; wired up below
pixie_fetch(zopfli
  https://github.com/google/zopfli/archive/refs/tags/zopfli-1.0.3.tar.gz
  e955a7739f71af37ef3349c4fa141c648e8775bceb2195be07e86f8e638814bd
  ${ZOPFLI_PATCH}
  SOURCE_SUBDIR _pixie_no_cmake)
file(GLOB ZOPFLI_SOURCES ${zopfli_SOURCE_DIR}/src/zopfli/*.c)
list(FILTER ZOPFLI_SOURCES EXCLUDE REGEX "zopfli_bin\\.c$")
add_library(pixie_png STATIC
  ${ZOPFLI_SOURCES}
  ${zopfli_SOURCE_DIR}/src/zopflipng/zopflipng_lib.cc
  ${lodepng_SOURCE_DIR}/lodepng.cpp
  ${lodepng_SOURCE_DIR}/lodepng_util.cpp)
get_filename_component(LODEPNG_PARENT ${lodepng_SOURCE_DIR} DIRECTORY)
target_include_directories(pixie_png PUBLIC
  ${zopfli_SOURCE_DIR}/src/zopflipng
  ${lodepng_SOURCE_DIR}
  ${LODEPNG_PARENT}
  ${zopfli_SOURCE_DIR}/src)
if(MSVC)
  target_compile_options(pixie_png PRIVATE /w)
else()
  target_compile_options(pixie_png PRIVATE -w)
endif()

# --- WebP: libwebp (lossy + lossless encode/decode, mux/demux for metadata and animation)
foreach(opt WEBP_BUILD_ANIM_UTILS WEBP_BUILD_CWEBP WEBP_BUILD_DWEBP WEBP_BUILD_GIF2WEBP
            WEBP_BUILD_IMG2WEBP WEBP_BUILD_VWEBP WEBP_BUILD_WEBPINFO WEBP_BUILD_WEBPMUX
            WEBP_BUILD_EXTRAS WEBP_BUILD_WEBP_JS WEBP_BUILD_FUZZTEST)
  set(${opt} OFF CACHE BOOL "" FORCE)
endforeach()
set(WEBP_BUILD_LIBWEBPMUX ON CACHE BOOL "" FORCE)
# libwebp's x86 SIMD paths don't build against Emscripten's SSE emulation; WASM uses the C paths.
if(EMSCRIPTEN)
  set(WEBP_ENABLE_SIMD OFF CACHE BOOL "" FORCE)
else()
  set(WEBP_ENABLE_SIMD ON CACHE BOOL "" FORCE)
endif()
pixie_fetch(libwebp
  https://github.com/webmproject/libwebp/archive/refs/tags/v1.6.0.tar.gz
  93a852c2b3efafee3723efd4636de855b46f9fe1efddd607e1f42f60fc8f2136)
add_library(pixie_webp INTERFACE)
target_link_libraries(pixie_webp INTERFACE webp webpdemux libwebpmux)
target_include_directories(pixie_webp INTERFACE ${libwebp_SOURCE_DIR}/src)

# --- GIF: giflib (decode, encode, and GifQuantizeBuffer for conversions)
pixie_fetch(giflib
  https://downloads.sourceforge.net/project/giflib/giflib-6.x/giflib-6.1.3.tar.gz
  b65b66b99f0424b93525f987386f22fc5efb9da2bfc92ad4a532249aaffbab0e
  SOURCE_SUBDIR _pixie_no_cmake)
add_library(pixie_gif STATIC
  ${giflib_SOURCE_DIR}/dgif_lib.c
  ${giflib_SOURCE_DIR}/egif_lib.c
  ${giflib_SOURCE_DIR}/gifalloc.c
  ${giflib_SOURCE_DIR}/gif_err.c
  ${giflib_SOURCE_DIR}/gif_hash.c
  ${giflib_SOURCE_DIR}/quantize.c
  ${giflib_SOURCE_DIR}/openbsd-reallocarray.c)
target_include_directories(pixie_gif PUBLIC ${giflib_SOURCE_DIR})
if(MSVC)
  target_compile_options(pixie_gif PRIVATE /w)
  target_compile_definitions(pixie_gif PRIVATE _CRT_SECURE_NO_WARNINGS)
else()
  target_compile_options(pixie_gif PRIVATE -w)
endif()

# --- TIFF: libtiff (LZW, PackBits, CCITT, ...) with zlib for Deflate, zstd for
# Zstandard, liblzma (xz) for LZMA, and this build's mozjpeg and libwebp for
# JPEG and WebP compression. zlib, zstd and liblzma are declared as the packages
# find_package() resolves to inside libtiff; JPEG and WebP are handed to its
# find modules as these targets. Each check libtiff would compile against a
# library not built yet is answered up front.
set(ZLIB_BUILD_SHARED OFF CACHE BOOL "" FORCE)
set(ZLIB_BUILD_STATIC ON CACHE BOOL "" FORCE)
set(ZLIB_BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(ZLIB_INSTALL OFF CACHE BOOL "" FORCE)
pixie_fetch(ZLIB
  https://github.com/madler/zlib/releases/download/v1.3.2/zlib-1.3.2.tar.gz
  bb329a0a2cd0274d05519d61c667c062e06990d72e125ee2dfa8de64f0119d16
  OVERRIDE_FIND_PACKAGE)
if(NOT TARGET ZLIB::ZLIB)
  add_library(ZLIB::ZLIB ALIAS zlibstatic)
endif()

foreach(opt ZSTD_BUILD_PROGRAMS ZSTD_BUILD_SHARED ZSTD_BUILD_TESTS ZSTD_BUILD_CONTRIB ZSTD_LEGACY_SUPPORT
            ZSTD_MULTITHREAD_SUPPORT ZSTD_BUILD_DICTBUILDER)
  set(${opt} OFF CACHE BOOL "" FORCE)
endforeach()
set(ZSTD_BUILD_STATIC ON CACHE BOOL "" FORCE)
pixie_fetch(ZSTD
  https://github.com/facebook/zstd/releases/download/v1.5.7/zstd-1.5.7.tar.gz
  eb33e51f49a15e023950cd7825ca74a4a2b43db8354825ac24fc1b7ee09e6fa3
  SOURCE_SUBDIR build/cmake
  OVERRIDE_FIND_PACKAGE)
add_library(ZSTD::ZSTD ALIAS libzstd_static)
set(ZSTD_HAVE_DECOMPRESS_STREAM ON CACHE BOOL "" FORCE)

foreach(opt XZ_TOOL_XZDEC XZ_TOOL_LZMADEC XZ_TOOL_LZMAINFO XZ_TOOL_XZ XZ_TOOL_SCRIPTS XZ_TOOL_SYMLINKS
            XZ_TOOL_SYMLINKS_LZMA XZ_DOC XZ_DOXYGEN XZ_NLS XZ_LZIP_DECODER XZ_MICROLZMA_ENCODER
            XZ_MICROLZMA_DECODER BUILD_TESTING)
  set(${opt} OFF CACHE BOOL "" FORCE)
endforeach()
set(XZ_THREADS no CACHE STRING "" FORCE)
set(XZ_SMALL ON CACHE BOOL "" FORCE)  # LZMA-compressed TIFFs are rare: size over speed
pixie_fetch(liblzma
  https://github.com/tukaani-project/xz/releases/download/v5.8.4/xz-5.8.4.tar.gz
  0014c7886930454fe8bd4228665b51af55eeae560ea135c9c4cd33f55b2591d9
  OVERRIDE_FIND_PACKAGE)
add_library(liblzma::liblzma ALIAS liblzma)
# Few TIFFs use these codecs, and the engine module is downloaded on every
# visit: in WebAssembly they are built for size (zstd with its own options
# for that), not speed.
if(EMSCRIPTEN)
  target_compile_options(libzstd_static PRIVATE -Oz)
  target_compile_definitions(libzstd_static PRIVATE ZSTD_NO_INLINE ZSTD_STRIP_ERROR_STRINGS
                             HUF_FORCE_DECOMPRESS_X1 ZSTD_FORCE_DECOMPRESS_SEQUENCES_SHORT)
  target_compile_options(liblzma PRIVATE -Oz)
endif()
# lzma.h is public only once installed; here it stays in the sources.
target_include_directories(liblzma INTERFACE $<BUILD_INTERFACE:${liblzma_SOURCE_DIR}/src/liblzma/api>)
set(LIBLZMA_FOUND TRUE)

add_library(JPEG::JPEG ALIAS pixie_jpeg)
set(JPEG_FOUND TRUE)
set(JPEG_LIBRARIES JPEG::JPEG)
set(HAVE_JPEGTURBO_DUAL_MODE_8 OFF CACHE BOOL "" FORCE)
set(HAVE_JPEGTURBO_DUAL_MODE_12 OFF CACHE BOOL "" FORCE)

add_library(WebP::webp ALIAS webp)
set(WebP_INCLUDE_DIR ${libwebp_SOURCE_DIR}/src CACHE PATH "" FORCE)
set(WebP_LIBRARY webp CACHE STRING "" FORCE)

# win32-io: tiffio.h gives its users the POSIX I/O layer's handle type on Windows
# too, so the library uses that layer as well and callbacks match their types.
# Old-style JPEG (compression 6), JBIG and LERC stay out, as do codecs from
# long-gone software (ThunderScan, NeXT, PixarLog, SGI LogLuv, Microsoft MDI).
foreach(opt tiff-tools tiff-tests tiff-contrib tiff-docs tiff-install tiff-cxx tiff-deprecated
            old-jpeg jpeg12 jbig lerc libdeflate ld-version-script win32-io thunder next pixarlog logluv mdi)
  set(${opt} OFF CACHE BOOL "" FORCE)
endforeach()
foreach(opt tiff-static zlib jpeg zstd lzma webp)
  set(${opt} ON CACHE BOOL "" FORCE)
endforeach()
pixie_fetch(libtiff
  https://download.osgeo.org/libtiff/tiff-4.7.2.tar.gz
  672bd7d10aee4606171afb864f3570b83340f6a33e2c186dc0512f7145ffdf6a
  ${LIBTIFF_PATCH})
add_library(pixie_tiff INTERFACE)
target_link_libraries(pixie_tiff INTERFACE tiff)

# --- HEIC: libheif with libde265 (HEVC decoding only, no plugins, no threads).
# Both are LGPL-licensed; the web app builds them into a module of their own,
# loaded only when a HEIC file needs it (see CMakeLists.txt).
if(PIXIE_HEIF)
  foreach(opt ENABLE_SDL ENABLE_DECODER ENABLE_ENCODER ENABLE_SHERLOCK265 ENABLE_INTERNAL_DEVELOPMENT_TOOLS
              WITH_FUZZERS)
    set(${opt} OFF CACHE BOOL "" FORCE)
  endforeach()
  # libde265's SIMD code is x86 SSE, which WebAssembly doesn't have.
  if(EMSCRIPTEN)
    set(ENABLE_SIMD OFF CACHE BOOL "" FORCE)
  endif()
  set(DE265_LOG_LEVEL none CACHE STRING "" FORCE)
  pixie_fetch(libde265
    https://github.com/strukturag/libde265/releases/download/v1.1.3/libde265-1.1.3.tar.gz
    554228bd17788c99a7e63b37ab5634722190e6e2bf60c1dcb01cef328e133905
    ${LIBDE265_PATCH})
  # de265.h is in the sources, the de265-version.h it includes in the build tree.
  target_include_directories(de265 INTERFACE $<BUILD_INTERFACE:${libde265_SOURCE_DIR}> $<BUILD_INTERFACE:${libde265_BINARY_DIR}>)
  target_compile_definitions(de265 INTERFACE LIBDE265_STATIC_BUILD)
  set(LIBDE265_INCLUDE_DIR ${libde265_BINARY_DIR} CACHE PATH "" FORCE)
  set(LIBDE265_LIBRARY de265 CACHE STRING "" FORCE)

  foreach(opt ENABLE_PLUGIN_LOADING ENABLE_MULTITHREADING_SUPPORT ENABLE_PARALLEL_TILE_DECODING
              WITH_X265 WITH_KVAZAAR WITH_UVG266 WITH_VVDEC WITH_VVENC WITH_X264 WITH_OpenH264_DECODER
              WITH_DAV1D WITH_AOM_DECODER WITH_AOM_ENCODER WITH_SvtEnc WITH_RAV1E WITH_JPEG_DECODER
              WITH_JPEG_ENCODER WITH_OpenJPEG_ENCODER WITH_OpenJPEG_DECODER WITH_FFMPEG_DECODER
              WITH_OPENJPH_ENCODER WITH_UNCOMPRESSED_CODEC WITH_WEBCODECS WITH_HEADER_COMPRESSION
              WITH_LIBSHARPYUV WITH_EXAMPLES WITH_EXAMPLE_HEIF_THUMB WITH_EXAMPLE_HEIF_VIEW WITH_GDK_PIXBUF
              BUILD_DOCUMENTATION WITH_LIBDE265_PLUGIN)
    set(${opt} OFF CACHE BOOL "" FORCE)
  endforeach()
  set(WITH_LIBDE265 ON CACHE BOOL "" FORCE)
  pixie_fetch(libheif
    https://github.com/strukturag/libheif/releases/download/v1.23.6/libheif-1.23.6.tar.gz
    4484346dc5995319dbc11e3a1c35d0a2ec46511ce370900869337fd2c7033125
    ${LIBHEIF_PATCH})
  target_compile_definitions(heif PUBLIC LIBHEIF_STATIC_BUILD)
  # libheif reports errors through C++ exceptions, which the WebAssembly build
  # otherwise leaves out: it gets WebAssembly's own exception handling, and so
  # does the HEIC module that links it (see CMakeLists.txt).
  if(EMSCRIPTEN)
    target_compile_options(heif PRIVATE -fexceptions -fwasm-exceptions)
  endif()
  add_library(pixie_libheif INTERFACE)
  target_link_libraries(pixie_libheif INTERFACE heif)
  # libheif/heif.h is in the sources, the libheif/heif_version.h it includes in the build tree.
  target_include_directories(pixie_libheif SYSTEM INTERFACE ${libheif_SOURCE_DIR}/libheif/api ${libheif_BINARY_DIR})
endif()

# --- Colour management: Little CMS, to convert to sRGB for formats that can't carry an ICC profile
pixie_fetch(lcms2
  https://github.com/mm2/Little-CMS/releases/download/lcms2.19.1/lcms2-2.19.1.tar.gz
  bfc54f7bab59fbc921012014a8032e4cba4abd46db47d46b76416a8c0b2815c8
  SOURCE_SUBDIR _pixie_no_cmake)
file(GLOB LCMS2_SOURCES ${lcms2_SOURCE_DIR}/src/*.c)
add_library(pixie_lcms STATIC ${LCMS2_SOURCES})
target_include_directories(pixie_lcms PUBLIC ${lcms2_SOURCE_DIR}/include)
# lcms2.h otherwise uses the "register" keyword, which C++17 removed.
target_compile_definitions(pixie_lcms PUBLIC CMS_NO_REGISTER_KEYWORD=1)
if(MSVC)
  target_compile_options(pixie_lcms PRIVATE /w)
  target_compile_definitions(pixie_lcms PRIVATE _CRT_SECURE_NO_WARNINGS)
else()
  target_compile_options(pixie_lcms PRIVATE -w)
endif()
# By design, Little CMS calls its interpolation routines through one generic
# function type (its plugin API relies on it); clang's UBSan check for that is off here.
if(CMAKE_C_COMPILER_ID MATCHES "Clang")
  target_compile_options(pixie_lcms PRIVATE -fno-sanitize=function)
endif()

# --- Decoding of BMP/TGA and BMP/TGA writing: stb_image, stb_image_write
pixie_fetch(stb
  https://github.com/nothings/stb/archive/2c980bb59875b0d32144a71867fbdebb2f77cd20.tar.gz
  9a955b1b49a4410088a2e0ee2a9c057c3c907d0c1d75454144cb980aca0ba515
  ${STB_PATCH}
  SOURCE_SUBDIR _pixie_no_cmake)
add_library(pixie_stb INTERFACE)
target_include_directories(pixie_stb SYSTEM INTERFACE ${stb_SOURCE_DIR})

# --- Meshes: meshoptimizer (welding, simplification, cache/overdraw/fetch ordering)
#     and its gltfpack (glTF/GLB optimiser, compiled in as a library; it brings
#     cgltf and fast_obj), plus miniply for PLY import.
pixie_fetch(meshoptimizer
  https://github.com/zeux/meshoptimizer/archive/refs/tags/v1.3.tar.gz
  ca4f47d451201593904b9e3067fa54b65db94bc34ab5173864950862b748a145
  ${MESHOPTIMIZER_PATCH})
pixie_fetch(miniply
  https://github.com/vilya/miniply/archive/1a235c70390fadf789695c9ccbf285ae712416b3.tar.gz
  81b0c3341706ee1b5fbdd50ab4d14fc3cec2486f9443014cf58c1d67cec31fa4
  ${MINIPLY_PATCH}
  SOURCE_SUBDIR _pixie_no_cmake)
set(GLTFPACK_DIR ${meshoptimizer_SOURCE_DIR}/gltf)
add_library(pixie_mesh_deps STATIC
  ${GLTFPACK_DIR}/animation.cpp
  ${GLTFPACK_DIR}/fileio.cpp
  ${GLTFPACK_DIR}/gltfpack.cpp
  ${GLTFPACK_DIR}/image.cpp
  ${GLTFPACK_DIR}/json.cpp
  ${GLTFPACK_DIR}/material.cpp
  ${GLTFPACK_DIR}/mesh.cpp
  ${GLTFPACK_DIR}/node.cpp
  ${GLTFPACK_DIR}/parseobj.cpp
  ${GLTFPACK_DIR}/parselib.cpp
  ${GLTFPACK_DIR}/parsegltf.cpp
  ${GLTFPACK_DIR}/stream.cpp
  ${GLTFPACK_DIR}/write.cpp
  ${miniply_SOURCE_DIR}/miniply.cpp)
# gltfpack is a command-line tool; its main() is renamed so gltfpack() can be called directly.
set_source_files_properties(${GLTFPACK_DIR}/gltfpack.cpp PROPERTIES COMPILE_DEFINITIONS "main=gltfpack_cli_main")
target_include_directories(pixie_mesh_deps PUBLIC
  ${meshoptimizer_SOURCE_DIR}/extern ${GLTFPACK_DIR} ${miniply_SOURCE_DIR})
target_link_libraries(pixie_mesh_deps PUBLIC meshoptimizer)
target_compile_definitions(pixie_mesh_deps PRIVATE GLTFPACK_NO_EXPERIMENTAL)
if(MSVC)
  target_compile_options(pixie_mesh_deps PRIVATE /w)
  target_compile_definitions(pixie_mesh_deps PRIVATE _CRT_SECURE_NO_WARNINGS)
else()
  target_compile_options(pixie_mesh_deps PRIVATE -w)
endif()

# --- Tests: doctest
if(PIXIE_BUILD_TESTS AND NOT EMSCRIPTEN)
  pixie_fetch(doctest
    https://github.com/doctest/doctest/archive/refs/tags/v2.5.3.tar.gz
    174ebc4e769928959614789c5b4e9c3d0a0f81a62bb608756b127bfebfb21331
    SOURCE_SUBDIR _pixie_no_cmake)
  add_library(doctest_header INTERFACE)
  target_include_directories(doctest_header INTERFACE ${doctest_SOURCE_DIR})
endif()
