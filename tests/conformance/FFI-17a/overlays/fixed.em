# The overlay of ../accept_fixed_spans_cross_as_arrays.em: each call asserted safe, and each span
# result's lifetime ([FFI-11]).

unsafe overlay cpp "fixed.hpp":
    fn sum3(values)
    fn zero2(values)
    fn new()
    fn cells() -> from(self)
    fn cells_mut() -> from(self)
