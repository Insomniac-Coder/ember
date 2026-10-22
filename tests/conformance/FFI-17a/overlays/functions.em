# The overlay of ../accept_a_std_function_crosses_as_a_callable.em: two `std::function` results as
# Ember callables (ODR-199); `make_kept`'s stays the object.

unsafe overlay cpp "functions.hpp":
    @ffi(std_function)
    fn make_adder(n)
    @ffi(std_function)
    fn make_mixer(weight)
    fn make_kept(n)
    fn live_guards()
