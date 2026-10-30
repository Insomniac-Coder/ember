# The overlay of ../accept_fixed_spans_cross_as_arrays.em: each call asserted safe, and each span
# result's lifetime ([FFI-11]).

unsafe overlay cpp "fixed.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn sum3(values)
    fn zero2(values)
    class Grid:
        fn new()
        @ffi(const)
        fn cells() -> from(self)
        fn cells_mut() -> from(self)
