# Third-party libraries, pinned to release tags. Every codec and mesh algorithm
# in Pixie Juice comes from one of these; src/ only orchestrates them.

include(FetchContent)
set(FETCHCONTENT_QUIET ON)
set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)

macro(pixie_fetch name repo tag)
  FetchContent_Declare(${name}
    GIT_REPOSITORY ${repo}
    GIT_TAG ${tag}
    GIT_SHALLOW TRUE
    # Populate only; each library is wired up explicitly below.
    SOURCE_SUBDIR _pixie_no_cmake)
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
  set(MOZJPEG_LIB ${MOZJPEG_BINARY}/libjpeg.a)
endif()
set(MOZJPEG_ARGS
  -DCMAKE_BUILD_TYPE=Release
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5
  -DCMAKE_POSITION_INDEPENDENT_CODE=ON
  "-DCMAKE_C_FLAGS=${CMAKE_C_FLAGS}"
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
endif()
ExternalProject_Add(mozjpeg_ep
  GIT_REPOSITORY https://github.com/mozilla/mozjpeg.git
  GIT_TAG v4.1.5
  GIT_SHALLOW TRUE
  SOURCE_DIR ${MOZJPEG_SOURCE}
  BINARY_DIR ${MOZJPEG_BINARY}
  CMAKE_ARGS ${MOZJPEG_ARGS}
  BUILD_COMMAND ${CMAKE_COMMAND} --build <BINARY_DIR> --target jpeg-static
  INSTALL_COMMAND ""
  BUILD_BYPRODUCTS ${MOZJPEG_LIB}
  UPDATE_DISCONNECTED TRUE)
# jpeglib.h/jmorecfg.h/jerror.h live in the source tree, jconfig.h in the build tree.
file(MAKE_DIRECTORY ${MOZJPEG_SOURCE} ${MOZJPEG_BINARY})
add_library(pixie_jpeg STATIC IMPORTED GLOBAL)
set_target_properties(pixie_jpeg PROPERTIES
  IMPORTED_LOCATION ${MOZJPEG_LIB}
  INTERFACE_INCLUDE_DIRECTORIES "${MOZJPEG_SOURCE};${MOZJPEG_BINARY}")
add_dependencies(pixie_jpeg mozjpeg_ep)

# --- PNG: zopflipng (lossless PNG optimiser built on lodepng + zopfli deflate)
pixie_fetch(zopfli https://github.com/google/zopfli.git zopfli-1.0.3)
file(GLOB ZOPFLI_SOURCES ${zopfli_SOURCE_DIR}/src/zopfli/*.c)
list(FILTER ZOPFLI_SOURCES EXCLUDE REGEX "zopfli_bin\\.c$")
add_library(pixie_png STATIC
  ${ZOPFLI_SOURCES}
  ${zopfli_SOURCE_DIR}/src/zopflipng/zopflipng_lib.cc
  ${zopfli_SOURCE_DIR}/src/zopflipng/lodepng/lodepng.cpp
  ${zopfli_SOURCE_DIR}/src/zopflipng/lodepng/lodepng_util.cpp)
target_include_directories(pixie_png PUBLIC
  ${zopfli_SOURCE_DIR}/src/zopflipng
  ${zopfli_SOURCE_DIR}/src/zopflipng/lodepng
  ${zopfli_SOURCE_DIR}/src)
if(MSVC)
  target_compile_options(pixie_png PRIVATE /w)
else()
  target_compile_options(pixie_png PRIVATE -w)
endif()

# --- WebP: libwebp (lossy + lossless encode/decode, mux/demux for metadata and animation)
FetchContent_Declare(libwebp
  GIT_REPOSITORY https://github.com/webmproject/libwebp.git
  GIT_TAG v1.6.0
  GIT_SHALLOW TRUE)
foreach(opt WEBP_BUILD_ANIM_UTILS WEBP_BUILD_CWEBP WEBP_BUILD_DWEBP WEBP_BUILD_GIF2WEBP
            WEBP_BUILD_IMG2WEBP WEBP_BUILD_VWEBP WEBP_BUILD_WEBPINFO WEBP_BUILD_WEBPMUX
            WEBP_BUILD_EXTRAS WEBP_BUILD_WEBP_JS WEBP_BUILD_FUZZTEST WEBP_BUILD_LIBWEBPMUX_NOT_USED)
  set(${opt} OFF CACHE BOOL "" FORCE)
endforeach()
set(WEBP_BUILD_LIBWEBPMUX ON CACHE BOOL "" FORCE)
# libwebp's x86 SIMD paths don't build against Emscripten's SSE emulation; WASM uses the C paths.
if(EMSCRIPTEN)
  set(WEBP_ENABLE_SIMD OFF CACHE BOOL "" FORCE)
else()
  set(WEBP_ENABLE_SIMD ON CACHE BOOL "" FORCE)
endif()
FetchContent_MakeAvailable(libwebp)
add_library(pixie_webp INTERFACE)
target_link_libraries(pixie_webp INTERFACE webp webpdemux libwebpmux)
target_include_directories(pixie_webp INTERFACE ${libwebp_SOURCE_DIR}/src)

# --- GIF: giflib (decode, encode, and GifQuantizeBuffer for conversions)
FetchContent_Declare(giflib
  GIT_REPOSITORY https://git.code.sf.net/p/giflib/code
  GIT_TAG 6.1.3
  GIT_SHALLOW TRUE
  SOURCE_SUBDIR _pixie_no_cmake)
FetchContent_MakeAvailable(giflib)
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

# --- TIFF: libtiff (LZW, Deflate, PackBits, CCITT, ...) with zlib for Deflate.
# zlib is declared as the package find_package(ZLIB) resolves to inside libtiff.
FetchContent_Declare(ZLIB
  GIT_REPOSITORY https://github.com/madler/zlib.git
  GIT_TAG v1.3.2
  GIT_SHALLOW TRUE
  OVERRIDE_FIND_PACKAGE)
set(ZLIB_BUILD_SHARED OFF CACHE BOOL "" FORCE)
set(ZLIB_BUILD_STATIC ON CACHE BOOL "" FORCE)
set(ZLIB_BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(ZLIB_INSTALL OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(ZLIB)
if(NOT TARGET ZLIB::ZLIB)
  add_library(ZLIB::ZLIB ALIAS zlibstatic)
endif()

FetchContent_Declare(libtiff
  GIT_REPOSITORY https://gitlab.com/libtiff/libtiff.git
  GIT_TAG v4.7.2
  GIT_SHALLOW TRUE)
foreach(opt tiff-tools tiff-tests tiff-contrib tiff-docs tiff-install tiff-cxx tiff-deprecated
            jpeg old-jpeg jpeg12 jbig lerc lzma zstd webp libdeflate ld-version-script)
  set(${opt} OFF CACHE BOOL "" FORCE)
endforeach()
set(tiff-static ON CACHE BOOL "" FORCE)
set(zlib ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(libtiff)
add_library(pixie_tiff INTERFACE)
target_link_libraries(pixie_tiff INTERFACE tiff)

# --- Colour management: Little CMS, to convert to sRGB for formats that can't carry an ICC profile
pixie_fetch(lcms2 https://github.com/mm2/Little-CMS.git lcms2.19.1)
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

# --- Decoding of BMP/TGA and BMP/TGA writing: stb_image, stb_image_write
pixie_fetch(stb https://github.com/nothings/stb.git 2c980bb59875b0d32144a71867fbdebb2f77cd20)
add_library(pixie_stb INTERFACE)
target_include_directories(pixie_stb SYSTEM INTERFACE ${stb_SOURCE_DIR})

# --- Meshes: meshoptimizer (welding, simplification, cache/overdraw/fetch ordering)
#     and its gltfpack (glTF/GLB optimiser, compiled in as a library; it brings
#     cgltf and fast_obj), plus miniply for PLY import.
FetchContent_Declare(meshoptimizer
  GIT_REPOSITORY https://github.com/zeux/meshoptimizer.git
  GIT_TAG v1.3
  GIT_SHALLOW TRUE)
FetchContent_MakeAvailable(meshoptimizer)
pixie_fetch(miniply https://github.com/vilya/miniply.git 1a235c70390fadf789695c9ccbf285ae712416b3)
set(GLTFPACK_DIR ${meshoptimizer_SOURCE_DIR}/gltf)
add_library(pixie_mesh_deps STATIC
  ${GLTFPACK_DIR}/animation.cpp
  ${GLTFPACK_DIR}/encodebasis.cpp
  ${GLTFPACK_DIR}/encodewebp.cpp
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
  pixie_fetch(doctest https://github.com/doctest/doctest.git v2.5.3)
  add_library(doctest_header INTERFACE)
  target_include_directories(doctest_header INTERFACE ${doctest_SOURCE_DIR})
endif()
