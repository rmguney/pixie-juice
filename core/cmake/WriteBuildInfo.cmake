# cmake -DOUT=... -DVERSION=... -DCORE=... -P WriteBuildInfo.cmake

# string(TIMESTAMP) honours SOURCE_DATE_EPOCH for reproducible builds.
string(TIMESTAMP stamp "%Y-%m-%dT%H:%M:%SZ" UTC)

include(${CMAKE_CURRENT_LIST_DIR}/SourceHash.cmake)
pixie_source_hash(${CORE} source)

file(WRITE ${OUT} "#pragma once\n\nnamespace pixie::build {\ninline constexpr const char* version = \"${VERSION}\";\ninline constexpr const char* timestamp = \"${stamp}\";\ninline constexpr const char* source = \"${source}\";\n}\n")
