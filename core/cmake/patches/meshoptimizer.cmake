# meshoptimizer's bundled fast_obj: an OBJ index with more digits than an int
# holds overflowed while parsing (undefined behaviour that UBSan reports). It
# now saturates at INT_MAX, which the readers reject as out of range.

include(${CMAKE_CURRENT_LIST_DIR}/Patch.cmake)

pixie_patch(extern/fast_obj.h [=[
    int sign;
    int num;


    if (*ptr == '-')
    {
        sign = -1;
        ptr++;
    }
    else
    {
        sign = +1;
    }

    num = 0;
    while (is_digit(*ptr))
        num = 10 * num + (*ptr++ - '0');
]=] [=[
    int sign;
    int num;


    if (*ptr == '-')
    {
        sign = -1;
        ptr++;
    }
    else
    {
        sign = +1;
    }

    num = 0;
    while (is_digit(*ptr))
    {
        int digit = *ptr++ - '0';
        num = num > (2147483647 - digit) / 10 ? 2147483647 : 10 * num + digit;
    }
]=])
