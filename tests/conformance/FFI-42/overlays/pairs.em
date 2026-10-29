# The overlay of ../accept_operators_of_a_template_instantiation.em: each operator method asserted
# safe to call, which lets it implement its interface ([FFI-2]).

unsafe overlay cpp "pairs.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Pair_int:
        fn add(other)
        fn eq(other)
        fn sum()
