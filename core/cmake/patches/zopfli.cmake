# zopfli:
# - Its bundled lodepng (20191107) is removed: upstream lodepng replaces it (see
#   Dependencies.cmake), and with the copy gone zopflipng's "lodepng/lodepng.h"
#   includes find that one.
# - Match bytes are compared with memcmp instead of being loaded through size_t
#   and unsigned int pointers, which are misaligned at most positions (undefined
#   behaviour that UBSan reports). Compilers turn these memcmp calls into the
#   same single loads.

include(${CMAKE_CURRENT_LIST_DIR}/Patch.cmake)

file(REMOVE_RECURSE src/zopflipng/lodepng)

pixie_patch(src/zopfli/lz77.c [=[
#include <stdlib.h>
]=] [=[
#include <stdlib.h>
#include <string.h>
]=])

pixie_patch(src/zopfli/lz77.c [=[
    while (scan < safe_end && *((size_t*)scan) == *((size_t*)match)) {
]=] [=[
    while (scan < safe_end && memcmp(scan, match, 8) == 0) {
]=])

pixie_patch(src/zopfli/lz77.c [=[
    while (scan < safe_end
        && *((unsigned int*)scan) == *((unsigned int*)match)) {
]=] [=[
    while (scan < safe_end && memcmp(scan, match, 4) == 0) {
]=])
