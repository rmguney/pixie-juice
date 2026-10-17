# libtiff with mozjpeg: mozjpeg's defaults make progressive JPEG, which TIFF
# readers don't expect in JPEG-compressed TIFFs. The scan script is cleared
# after the defaults are set, as mozjpeg's own cjpeg -baseline does, so strips
# are baseline (trellis quantisation stays).

include(${CMAKE_CURRENT_LIST_DIR}/Patch.cmake)

pixie_patch(libtiff/tif_jpeg.c
"static int TIFFjpeg_set_defaults(JPEGState *sp)
{
    return CALLVJPEG(sp, jpeg_set_defaults(&sp->cinfo.c));
}"
"static int TIFFjpeg_set_defaults(JPEGState *sp)
{
    if (!CALLVJPEG(sp, jpeg_set_defaults(&sp->cinfo.c)))
        return 0;
    sp->cinfo.c.num_scans = 0; /* baseline, as cjpeg -baseline */
    sp->cinfo.c.scan_info = NULL;
    return 1;
}")
