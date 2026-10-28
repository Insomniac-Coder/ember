# The overlay of ../accept_a_newtype_is_nominal_with_its_typedefs_representation.em: three typedefs
# made nominal ([FFI-8], ODR-249), `count_t` left an alias, and each function asserted safe.

unsafe overlay c "newtypes.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(newtype)
    type fd_t
    @ffi(newtype)
    type seconds_t
    @ffi(newtype)
    type widget_ref
    fn open_slot(number)
    fn slot_number(fd)
    fn later(start, by)
    fn twice(n)
