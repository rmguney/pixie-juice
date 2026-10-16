# A hash of the sources the engine is built from (core/ without its tests),
# with line endings normalized so a Windows checkout and CI agree. The
# WebAssembly module embeds it, and CI checks that the committed module carries
# the hash of the committed sources.
#
#   cmake -DCORE=<core dir> -P SourceHash.cmake    prints the hash

function(pixie_source_hash core out)
  get_filename_component(core ${core} ABSOLUTE)
  file(GLOB_RECURSE files RELATIVE ${core}
    ${core}/CMakeLists.txt ${core}/CMakePresets.json ${core}/cmake/* ${core}/include/* ${core}/src/*)
  list(SORT files)
  set(manifest "")
  foreach(file IN LISTS files)
    file(READ ${core}/${file} content)
    string(REPLACE "\r\n" "\n" content "${content}")
    string(SHA256 hash "${content}")
    string(APPEND manifest "${file} ${hash}\n")
  endforeach()
  string(SHA256 hash "${manifest}")
  set(${out} ${hash} PARENT_SCOPE)
endfunction()

if(CMAKE_SCRIPT_MODE_FILE STREQUAL CMAKE_CURRENT_LIST_FILE)
  pixie_source_hash(${CORE} hash)
  execute_process(COMMAND ${CMAKE_COMMAND} -E echo ${hash})
endif()
