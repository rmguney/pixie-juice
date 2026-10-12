# cmake -DOUT=... -DVERSION=... -DREPO=... -P WriteBuildInfo.cmake

# string(TIMESTAMP) honours SOURCE_DATE_EPOCH for reproducible builds.
string(TIMESTAMP stamp "%Y-%m-%dT%H:%M:%SZ" UTC)

execute_process(COMMAND git rev-parse --short HEAD
  WORKING_DIRECTORY ${REPO} OUTPUT_VARIABLE commit
  OUTPUT_STRIP_TRAILING_WHITESPACE ERROR_QUIET RESULT_VARIABLE rc)
if(NOT rc EQUAL 0 OR commit STREQUAL "")
  set(commit "unknown")
else()
  # Ignore the generated web module itself, which is rebuilt from the tree.
  execute_process(COMMAND git status --porcelain --untracked-files=no -- . ":!web/src/core/generated"
    WORKING_DIRECTORY ${REPO} OUTPUT_VARIABLE dirty
    OUTPUT_STRIP_TRAILING_WHITESPACE ERROR_QUIET)
  if(NOT dirty STREQUAL "")
    set(commit "${commit}-dirty")
  endif()
endif()

file(WRITE ${OUT} "#pragma once\n\nnamespace pixie::build {\ninline constexpr const char* version = \"${VERSION}\";\ninline constexpr const char* commit = \"${commit}\";\ninline constexpr const char* timestamp = \"${stamp}\";\n}\n")
