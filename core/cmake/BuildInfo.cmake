# Generates generated/pixie/build_info.hpp on every build, so build_commit() and
# build_timestamp() always describe the binary that was actually produced.

set(PIXIE_BUILD_INFO_HEADER ${CMAKE_CURRENT_BINARY_DIR}/generated/pixie/build_info.hpp)

add_custom_target(pixie_build_info
  COMMAND ${CMAKE_COMMAND}
    -DOUT=${PIXIE_BUILD_INFO_HEADER}
    -DVERSION=${PROJECT_VERSION}
    -DREPO=${CMAKE_CURRENT_SOURCE_DIR}/..
    -P ${CMAKE_CURRENT_LIST_DIR}/WriteBuildInfo.cmake
  BYPRODUCTS ${PIXIE_BUILD_INFO_HEADER}
  COMMENT "Writing build info"
  VERBATIM)
