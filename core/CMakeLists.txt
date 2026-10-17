cmake_minimum_required(VERSION 3.25)

project(pixie_core VERSION 0.2.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_C_STANDARD 11)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

option(PIXIE_BUILD_TESTS "Build the native unit tests" ON)
option(PIXIE_SANITIZE "Build with AddressSanitizer (and UBSan where available)" OFF)
option(PIXIE_HEIF "Build the HEIC decoder (libheif and libde265, both LGPL)" ON)
option(PIXIE_FUZZ "Build the libFuzzer harness (clang only; see fuzz/README.md)" OFF)

# Sanitizer builds instrument every library too, and any report stops the program.
# Fuzzing builds also give every library coverage instrumentation for libFuzzer.
if(PIXIE_FUZZ)
  if(NOT CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    message(FATAL_ERROR "PIXIE_FUZZ needs clang (libFuzzer)")
  endif()
  add_compile_options(-fsanitize=fuzzer-no-link)
endif()
if(PIXIE_SANITIZE)
  if(MSVC)
    add_compile_options(/fsanitize=address /Zi)
  else()
    add_compile_options(-fsanitize=address,undefined -fno-sanitize-recover=undefined -fno-omit-frame-pointer)
    add_link_options(-fsanitize=address,undefined)
  endif()
endif()

# Settings for Pixie Juice's own code; the bundled libraries build with their
# defaults. Warnings are errors; native builds get stack protection, fortified
# libc calls (optimised builds) and libstdc++ bounds assertions.
function(pixie_project_options target)
  if(WIN32)
    target_compile_definitions(${target} PRIVATE _CRT_SECURE_NO_WARNINGS)  # fopen is portable C
  endif()
  if(MSVC)
    target_compile_options(${target} PRIVATE /W4 /WX /permissive- /utf-8)
  else()
    target_compile_options(${target} PRIVATE -Wall -Wextra -Wpedantic -Werror)
    if(NOT EMSCRIPTEN)
      target_compile_options(${target} PRIVATE -fstack-protector-strong
                             "$<$<NOT:$<CONFIG:Debug>>:-U_FORTIFY_SOURCE;-D_FORTIFY_SOURCE=3>")
      target_compile_definitions(${target} PRIVATE _GLIBCXX_ASSERTIONS)
    endif()
  endif()
endfunction()

include(cmake/Dependencies.cmake)
include(cmake/BuildInfo.cmake)

add_library(pixie_core STATIC
  src/format.cpp
  src/pipeline.cpp
  src/verify.cpp
  src/image/decode.cpp
  src/image/convert.cpp
  src/image/jpeg.cpp
  src/image/png.cpp
  src/image/webp.cpp
  src/image/gif.cpp
  src/image/ico.cpp
  src/image/tiff.cpp
  src/image/metadata.cpp
  src/mesh/mesh_io.cpp
  src/mesh/mesh_optimize.cpp
  src/mesh/gltf.cpp
  src/third_party_impl.cpp
)
target_include_directories(pixie_core
  PUBLIC include
  PRIVATE src)
target_link_libraries(pixie_core PRIVATE
  pixie_jpeg pixie_png pixie_webp pixie_gif pixie_tiff pixie_lcms pixie_mesh_deps pixie_stb ZLIB::ZLIB)
pixie_project_options(pixie_core)

# HEIC decoding (include/pixie/heif.hpp), apart from the core: see Dependencies.cmake.
if(PIXIE_HEIF)
  add_library(pixie_heif STATIC src/heif/heif.cpp)
  target_include_directories(pixie_heif PUBLIC include PRIVATE src)
  target_link_libraries(pixie_heif PRIVATE pixie_libheif pixie_lcms)
  pixie_project_options(pixie_heif)
endif()

# The settings both WebAssembly modules share.
function(pixie_wasm_module target output export)
  set_target_properties(${target} PROPERTIES OUTPUT_NAME ${output} SUFFIX ".js")
  target_include_directories(${target} PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/generated)
  add_dependencies(${target} pixie_build_info)
  pixie_project_options(${target})
  target_link_options(${target} PRIVATE
    -lembind
    -sMODULARIZE=1
    -sEXPORT_ES6=1
    -sEXPORT_NAME=${export}
    -sENVIRONMENT=web,worker
    -sALLOW_MEMORY_GROWTH=1
    -sMAXIMUM_MEMORY=2GB
    -sSTACK_SIZE=1MB
    -sINCOMING_MODULE_JS_API=[locateFile,wasmBinary,print,printErr]
    --emit-tsd ${output}.d.ts
  )
  # Copy the generated module into the web app, which ships it as-is.
  set(web_out "${CMAKE_CURRENT_SOURCE_DIR}/../web/src/core/generated")
  add_custom_command(TARGET ${target} POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory "${web_out}"
    COMMAND ${CMAKE_COMMAND} -E copy
      "$<TARGET_FILE_DIR:${target}>/${output}.js"
      "$<TARGET_FILE_DIR:${target}>/${output}.wasm"
      "$<TARGET_FILE_DIR:${target}>/${output}.d.ts"
      "${web_out}"
    VERBATIM)
endfunction()

if(EMSCRIPTEN)
  add_executable(pixie_wasm src/bindings/embind.cpp)
  target_link_libraries(pixie_wasm PRIVATE pixie_core)
  pixie_wasm_module(pixie_wasm pixie_core createPixieCore)
  # Every visit downloads this module: the link-time pass optimizes it for size
  # (the code itself stays compiled for speed).
  target_link_options(pixie_wasm PRIVATE -Os)
  # The HEIC module, which the web app loads when a HEIC file needs it.
  if(PIXIE_HEIF)
    add_executable(pixie_heif_wasm src/bindings/heif_embind.cpp)
    target_link_libraries(pixie_heif_wasm PRIVATE pixie_heif)
    pixie_wasm_module(pixie_heif_wasm pixie_heif createPixieHeif)
    target_link_options(pixie_heif_wasm PRIVATE -fwasm-exceptions)  # for libheif
  endif()
endif()

if(NOT EMSCRIPTEN)
  # The C API (include/pixie/pixie.h) as a shared library for other languages:
  # pixie.dll, libpixie.so or libpixie.dylib, exporting only the pixie_* functions.
  add_library(pixie SHARED src/bindings/c_api.cpp)
  target_link_libraries(pixie PRIVATE pixie_core)
  target_include_directories(pixie PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/generated)
  target_compile_definitions(pixie PRIVATE PIXIE_BUILDING_SHARED)
  pixie_project_options(pixie)
  add_dependencies(pixie pixie_build_info)
  set_target_properties(pixie PROPERTIES
    C_VISIBILITY_PRESET hidden
    CXX_VISIBILITY_PRESET hidden
    VISIBILITY_INLINES_HIDDEN ON
    VERSION ${PROJECT_VERSION}
    SOVERSION ${PROJECT_VERSION_MAJOR})
  # The bundled libraries' symbols stay private (some mark their APIs visible).
  if(APPLE)
    target_link_options(pixie PRIVATE "LINKER:-exported_symbol,_pixie_*")
  elseif(NOT WIN32)
    target_link_options(pixie PRIVATE -Wl,--exclude-libs,ALL)
  endif()
endif()

if(PIXIE_FUZZ)
  add_executable(pixie_fuzz fuzz/fuzz_pixie.cpp)
  target_link_libraries(pixie_fuzz PRIVATE pixie_core)
  target_link_options(pixie_fuzz PRIVATE -fsanitize=fuzzer)
  add_executable(pixie_fuzz_seeds fuzz/seeds.cpp)
  target_link_libraries(pixie_fuzz_seeds PRIVATE pixie_core pixie_lcms)
  target_include_directories(pixie_fuzz_seeds PRIVATE src)
  target_link_options(pixie_fuzz_seeds PRIVATE -fsanitize=fuzzer-no-link)
  if(PIXIE_HEIF)
    target_link_libraries(pixie_fuzz PRIVATE pixie_heif)
    target_compile_definitions(pixie_fuzz PRIVATE PIXIE_FUZZ_HEIF)
    target_compile_definitions(pixie_fuzz_seeds PRIVATE "PIXIE_HEIF_TEST_DATA=\"${libheif_SOURCE_DIR}/tests/data\"")
  endif()
  pixie_project_options(pixie_fuzz)
  pixie_project_options(pixie_fuzz_seeds)
endif()

if(PIXIE_BUILD_TESTS AND NOT EMSCRIPTEN)
  enable_testing()
  add_executable(pixie_tests
    tests/main.cpp
    tests/test_format.cpp
    tests/test_image.cpp
    tests/test_mesh.cpp
    tests/test_robustness.cpp
    tests/test_regressions.cpp
    tests/test_review.cpp
    tests/test_c_api.cpp
    tests/c_api.c
    src/bindings/c_api.cpp
  )
  # Tests build fixtures with the same libraries the core wraps.
  find_package(Threads REQUIRED)
  target_link_libraries(pixie_tests PRIVATE
    pixie_core pixie_png pixie_jpeg pixie_gif pixie_tiff pixie_lcms pixie_mesh_deps doctest_header
    ZLIB::ZLIB Threads::Threads)
  target_include_directories(pixie_tests PRIVATE src ${CMAKE_CURRENT_BINARY_DIR}/generated)
  add_dependencies(pixie_tests pixie_build_info)
  if(PIXIE_HEIF)
    # HEIC samples from libheif's own tests.
    target_sources(pixie_tests PRIVATE tests/test_heif.cpp)
    target_link_libraries(pixie_tests PRIVATE pixie_heif)
    target_compile_definitions(pixie_tests PRIVATE "PIXIE_HEIF_TEST_DATA=\"${libheif_SOURCE_DIR}/tests/data\"")
  endif()
  add_test(NAME pixie_tests COMMAND pixie_tests)
endif()
