# mozjpeg: free the per-scan output buffers of an aborted compression.
#
# With scan optimization, each candidate scan is written to its own malloc'd
# buffer (jcmaster.c), freed only once the scans have been chosen. An error
# exit in between left them allocated: jpeg_destroy_compress releases the
# memory pools, not these. The buffers now go with the object: jpeg_abort and
# jpeg_destroy free them. For that, the memory destination keeps the caller's
# pointer on the live buffer when it grows (until now it pointed at the freed
# one until term_destination), and freed buffers are cleared.

include(${CMAKE_CURRENT_LIST_DIR}/Patch.cmake)

pixie_patch(jdatadst.c [=[
  dest->buffer = nextbuffer;
  dest->bufsize = nextsize;
]=] [=[
  dest->buffer = nextbuffer;
  dest->bufsize = nextsize;
  *dest->outbuffer = nextbuffer;
]=])

pixie_patch(jcmaster.c [=[
    for (i = 0; i < cinfo->num_scans; i++)
      if (master->scan_buffer[i])
        free(master->scan_buffer[i]);
  }
}
]=] [=[
    for (i = 0; i < cinfo->num_scans; i++) {
      free(master->scan_buffer[i]);
      master->scan_buffer[i] = NULL;
    }
  }
}

/*
 * Free the scan buffers of a compression that did not finish.
 */

GLOBAL(void)
jfree_scan_buffers(j_compress_ptr cinfo)
{
  my_master_ptr master = (my_master_ptr)cinfo->master;
  int i;

  if (master == NULL)
    return;
  for (i = 0; i < 64; i++) {
    free(master->scan_buffer[i]);
    master->scan_buffer[i] = NULL;
  }
}
]=])

pixie_patch(jpegint.h [=[
EXTERN(void) jinit_c_master_control (j_compress_ptr cinfo,
]=] [=[
EXTERN(void) jfree_scan_buffers(j_compress_ptr cinfo);
EXTERN(void) jinit_c_master_control (j_compress_ptr cinfo,
]=])

pixie_patch(jcomapi.c [=[
  if (cinfo->mem == NULL)
    return;

  /* Releasing pools]=] [=[
  if (cinfo->mem == NULL)
    return;

  if (!cinfo->is_decompressor)
    jfree_scan_buffers((j_compress_ptr)cinfo);

  /* Releasing pools]=])

pixie_patch(jcomapi.c [=[
  if (cinfo->mem != NULL)
    (*cinfo->mem->self_destruct) (cinfo);
]=] [=[
  if (cinfo->mem != NULL && !cinfo->is_decompressor)
    jfree_scan_buffers((j_compress_ptr)cinfo);
  if (cinfo->mem != NULL)
    (*cinfo->mem->self_destruct) (cinfo);
]=])
