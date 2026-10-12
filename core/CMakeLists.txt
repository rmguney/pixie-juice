cmake_minimum_required(VERSION 3.25)

project(pixie_core VERSION 0.2.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_C_STANDARD 11)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

option(PIXIE_BUILD_TESTS "Build the native unit tests" ON)
option(PIXIE_SANITIZE "Build with AddressSanitizer (and UBSan where available)" OFF)

if(PIXIE_SANITIZE)
  if(MSVC)
    add_compile_options(/fsanitize=address /Zi)
  else()
    add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer)
    add_link_options(-fsanitize=address,undefined)
  endif()
endif()

# Warnings for Pixie Juice's own code (the bundled libraries build with their defaults).
function(pixie_warnings target)
  if(MSVC)
    target_compile_options(${target} PRIVATE /W4 /permissive- /utf-8)
    target_compile_definitions(${target} PRIVATE _CRT_SECURE_NO_WARNINGS)  # fopen is portable C
  else()
    target_compile_options(${target} PRIVATE -Wall -Wextra -Wpedantic)
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
  PRIVATE src ${CMAKE_CURRENT_BINARY_DIR}/generated)
target_link_libraries(pixie_core PRIVATE
  pixie_jpeg pixie_png pixie_webp pixie_gif pixie_tiff pixie_lcms pixie_mesh_deps pixie_stb)
pixie_warnings(pixie_core)

if(EMSCRIPTEN)
  add_executable(pixie_wasm src/bindings/embind.cpp)
  target_link_libraries(pixie_wasm PRIVATE pixie_core)
  pixie_warnings(pixie_wasm)
  target_include_directories(pixie_wasm PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/generated)
  add_dependencies(pixie_wasm pixie_build_info)
  set_target_properties(pixie_wasm PROPERTIES OUTPUT_NAME "pixie_core" SUFFIX ".js")
  target_link_options(pixie_wasm PRIVATE
    -lembind
    -sMODULARIZE=1
    -sEXPORT_ES6=1
    -sEXPORT_NAME=createPixieCore
    -sENVIRONMENT=web,worker
    -sALLOW_MEMORY_GROWTH=1
    -sMAXIMUM_MEMORY=2GB
    -sSTACK_SIZE=1MB
    -sINCOMING_MODULE_JS_API=[locateFile,wasmBinary,print,printErr]
    --emit-tsd pixie_core.d.ts
  )
  # Copy the generated module into the web app, which ships it as-is.
  set(PIXIE_WEB_OUT "${CMAKE_CURRENT_SOURCE_DIR}/../web/src/core/generated")
  add_custom_command(TARGET pixie_wasm POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory "${PIXIE_WEB_OUT}"
    COMMAND ${CMAKE_COMMAND} -E copy
      "$<TARGET_FILE_DIR:pixie_wasm>/pixie_core.js"
      "$<TARGET_FILE_DIR:pixie_wasm>/pixie_core.wasm"
      "$<TARGET_FILE_DIR:pixie_wasm>/pixie_core.d.ts"
      "${PIXIE_WEB_OUT}"
    VERBATIM)
endif()

if(NOT EMSCRIPTEN)
  # The C API (include/pixie/pixie.h) as a shared library for other languages:
  # pixie.dll, libpixie.so or libpixie.dylib, exporting only the pixie_* functions.
  add_library(pixie SHARED src/bindings/c_api.cpp)
  target_link_libraries(pixie PRIVATE pixie_core)
  target_include_directories(pixie PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/generated)
  target_compile_definitions(pixie PRIVATE PIXIE_BUILDING_SHARED)
  pixie_warnings(pixie)
  add_dependencies(pixie pixie_build_info)
  set_target_properties(pixie PROPERTIES
    C_VISIBILITY_PRESET hidden
    CXX_VISIBILITY_PRESET hidden
    VISIBILITY_INLINES_HIDDEN ON
    VERSION ${PROJECT_VERSION}
    SOVERSION ${PROJECT_VERSION_MAJOR})
  if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    target_link_options(pixie PRIVATE -Wl,--exclude-libs,ALL)  # keep the bundled libraries' symbols private
  endif()
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
    tests/test_c_api.cpp
    tests/c_api.c
    src/bindings/c_api.cpp
  )
  # Tests build fixtures with the same libraries the core wraps.
  find_package(Threads REQUIRED)
  target_link_libraries(pixie_tests PRIVATE
    pixie_core pixie_stb pixie_png pixie_jpeg pixie_webp pixie_gif pixie_tiff pixie_lcms pixie_mesh_deps doctest_header
    Threads::Threads)
  target_include_directories(pixie_tests PRIVATE src ${CMAKE_CURRENT_BINARY_DIR}/generated)
  add_dependencies(pixie_tests pixie_build_info)
  add_test(NAME pixie_tests COMMAND pixie_tests)
endif()
