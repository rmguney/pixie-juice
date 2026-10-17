# Checks generated/pixie/build_info.hpp on every build, so the source hash always
# describes the binary produced; it is rewritten only when the source changes.

set(PIXIE_BUILD_INFO_HEADER ${CMAKE_CURRENT_BINARY_DIR}/generated/pixie/build_info.hpp)

add_custom_target(pixie_build_info
  COMMAND ${CMAKE_COMMAND}
    -DOUT=${PIXIE_BUILD_INFO_HEADER}
    -DVERSION=${PROJECT_VERSION}
    -DCORE=${CMAKE_CURRENT_SOURCE_DIR}
    -P ${CMAKE_CURRENT_LIST_DIR}/WriteBuildInfo.cmake
  BYPRODUCTS ${PIXIE_BUILD_INFO_HEADER}
  COMMENT "Writing build info"
  VERBATIM)
