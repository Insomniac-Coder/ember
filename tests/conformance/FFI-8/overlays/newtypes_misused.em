# The overlay of ../reject_a_newtype_entry_names_a_typedef_alone.em: `@ffi(newtype)` on a struct
# (no typedef), and with another word beside it ([FFI-8], ODR-249, [FFI-12]).

unsafe overlay c "newtypes.h":
    @ffi(newtype)
    type widget
    @ffi(newtype, handle)
    type fd_t
    fn twice(n)
