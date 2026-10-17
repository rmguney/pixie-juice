# libde265 as part of another build: its library looked for the config.h it
# generates in the top-level build directory, which is only its own when it is
# built alone.

include(${CMAKE_CURRENT_LIST_DIR}/Patch.cmake)

pixie_patch(libde265/CMakeLists.txt
  "target_include_directories(de265 PRIVATE \${CMAKE_BINARY_DIR} \${CMAKE_CURRENT_BINARY_DIR})"
  "target_include_directories(de265 PRIVATE \${PROJECT_BINARY_DIR} \${CMAKE_CURRENT_BINARY_DIR})")
