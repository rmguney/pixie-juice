# miniply: read binary property values with memcpy. Values sit at arbitrary
# offsets in the file, so loading them through typed pointers was misaligned
# (undefined behaviour that UBSan reports).

include(${CMAKE_CURRENT_LIST_DIR}/Patch.cmake)

pixie_patch(miniply.cpp [=[
  template <class T>
  static void copy_and_convert_to(T* dest, const uint8_t* src, PLYPropertyType srcType)
  {
    switch (srcType) {
    case PLYPropertyType::Char:   *dest = static_cast<T>(*reinterpret_cast<const int8_t*>(src)); break;
    case PLYPropertyType::UChar:  *dest = static_cast<T>(*reinterpret_cast<const uint8_t*>(src)); break;
    case PLYPropertyType::Short:  *dest = static_cast<T>(*reinterpret_cast<const int16_t*>(src)); break;
    case PLYPropertyType::UShort: *dest = static_cast<T>(*reinterpret_cast<const uint16_t*>(src)); break;
    case PLYPropertyType::Int:    *dest = static_cast<T>(*reinterpret_cast<const int*>(src)); break;
    case PLYPropertyType::UInt:   *dest = static_cast<T>(*reinterpret_cast<const uint32_t*>(src)); break;
    case PLYPropertyType::Float:  *dest = static_cast<T>(*reinterpret_cast<const float*>(src)); break;
    case PLYPropertyType::Double: *dest = static_cast<T>(*reinterpret_cast<const double*>(src)); break;
]=] [=[
  template <class V>
  static V load_unaligned(const uint8_t* src)
  {
    V value;
    std::memcpy(&value, src, sizeof(V));
    return value;
  }

  template <class T>
  static void copy_and_convert_to(T* dest, const uint8_t* src, PLYPropertyType srcType)
  {
    switch (srcType) {
    case PLYPropertyType::Char:   *dest = static_cast<T>(load_unaligned<int8_t>(src)); break;
    case PLYPropertyType::UChar:  *dest = static_cast<T>(load_unaligned<uint8_t>(src)); break;
    case PLYPropertyType::Short:  *dest = static_cast<T>(load_unaligned<int16_t>(src)); break;
    case PLYPropertyType::UShort: *dest = static_cast<T>(load_unaligned<uint16_t>(src)); break;
    case PLYPropertyType::Int:    *dest = static_cast<T>(load_unaligned<int>(src)); break;
    case PLYPropertyType::UInt:   *dest = static_cast<T>(load_unaligned<uint32_t>(src)); break;
    case PLYPropertyType::Float:  *dest = static_cast<T>(load_unaligned<float>(src)); break;
    case PLYPropertyType::Double: *dest = static_cast<T>(load_unaligned<double>(src)); break;
]=])
