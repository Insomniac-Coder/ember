# The overlay of ../accept_a_newtype_in_a_struct_crosses_by_value.em: the members' typedefs made
# nominal ([FFI-8], ODR-249), and each function asserted safe.

unsafe overlay c "stamps.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(newtype)
    type when_t
    @ffi(newtype)
    type tag_t
    fn span_length(s)
    fn span_shift(s, by)
    fn stamp_tag(s)
    fn stamp_make(at, tag)
    fn lone_value(l)
    fn lone_twice(l)
