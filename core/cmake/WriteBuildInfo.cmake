# cmake -DOUT=... -DVERSION=... -DCORE=... -P WriteBuildInfo.cmake

# string(TIMESTAMP) honours SOURCE_DATE_EPOCH for reproducible builds.
string(TIMESTAMP stamp "%Y-%m-%dT%H:%M:%SZ" UTC)

include(${CMAKE_CURRENT_LIST_DIR}/SourceHash.cmake)
pixie_source_hash(${CORE} source)

# Unchanged source and version: the header (and its timestamp, when that source
# was first built) stays as it is, so nothing that includes it rebuilds.
if(EXISTS ${OUT})
  file(READ ${OUT} existing)
  string(FIND "${existing}" "version = \"${VERSION}\";" same_version)
  string(FIND "${existing}" "source = \"${source}\";" same_source)
  if(same_version GREATER -1 AND same_source GREATER -1)
    return()
  endif()
endif()

file(WRITE ${OUT} "#pragma once\n\nnamespace pixie::build {\ninline constexpr const char* version = \"${VERSION}\";\ninline constexpr const char* timestamp = \"${stamp}\";\ninline constexpr const char* source = \"${source}\";\n}\n")
