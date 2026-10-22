# The overlay of ../accept_operators_of_a_template_instantiation.em: each operator method asserted
# safe to call, which lets it implement its interface ([FFI-2]).

unsafe overlay cpp "pairs.hpp":
    fn add(other)
    fn eq(other)
    fn sum()
