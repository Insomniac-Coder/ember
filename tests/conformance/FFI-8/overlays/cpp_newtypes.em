# The overlay of ../accept_a_cpp_alias_is_a_newtype.em: a C++ `using` alias and a `typedef` made
# nominal ([FFI-8], ODR-249).

unsafe overlay cpp "cpp_newtypes.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(newtype)
    type port_t
    @ffi(newtype)
    type ratio_t
    fn next_port(port)
    fn halve(ratio)
    class Socket:
        fn new(port)
        fn port()
