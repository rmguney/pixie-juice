# stb_image_resize2: the SIMD linear-to-sRGB conversion looked values up through
# a pointer 912 entries before the start of its table, offsetting indices that
# start at 912. Forming that pointer is undefined behaviour (UBSan reports it);
# the lookups now index the table itself, minus the offset.

include(${CMAKE_CURRENT_LIST_DIR}/Patch.cmake)

pixie_patch_all(stb_image_resize2.h "( fp32_to_srgb8_tab4 - (127-13)*8 )" "( fp32_to_srgb8_tab4 )")
foreach(lane 0 1 2 3)
  foreach(vector 0 1 2 3)
    pixie_patch_all(stb_image_resize2.h
      "table[temp${vector}.m128i_i32[${lane}]]"
      "table[temp${vector}.m128i_i32[${lane}] - (127-13)*8]")
  endforeach()
endforeach()
