# Helpers for the patch scripts in this directory. Each script runs in a
# library's source directory after download (PATCH_COMMAND cmake -P ...).

# Replaces the one occurrence of <old> in <file> with <new>; line endings are
# normalized to \n first. Fails if <old> no longer matches (an upstream change),
# and does nothing if the patch is already in.
function(pixie_patch file old new)
  file(READ ${file} text)
  string(REPLACE "\r\n" "\n" text "${text}")
  string(REPLACE "\r\n" "\n" old "${old}")
  string(REPLACE "\r\n" "\n" new "${new}")
  string(FIND "${text}" "${new}" applied)
  if(NOT applied EQUAL -1)
    return()
  endif()
  string(FIND "${text}" "${old}" first)
  string(FIND "${text}" "${old}" last REVERSE)
  if(first EQUAL -1 OR NOT first EQUAL last)
    message(FATAL_ERROR "Patch for ${file} does not apply: the code it replaces changed upstream.")
  endif()
  string(REPLACE "${old}" "${new}" text "${text}")
  file(WRITE ${file} "${text}")
endfunction()

# Replaces every occurrence of <old> in <file> with <new> (which must not
# contain <old>). Fails if neither is there.
function(pixie_patch_all file old new)
  file(READ ${file} text)
  string(REPLACE "\r\n" "\n" text "${text}")
  string(FIND "${text}" "${old}" found)
  if(found EQUAL -1)
    string(FIND "${text}" "${new}" applied)
    if(applied EQUAL -1)
      message(FATAL_ERROR "Patch for ${file} does not apply: the code it replaces changed upstream.")
    endif()
    return()
  endif()
  string(REPLACE "${old}" "${new}" text "${text}")
  file(WRITE ${file} "${text}")
endfunction()
