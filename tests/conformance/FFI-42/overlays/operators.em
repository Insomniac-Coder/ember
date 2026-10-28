# The overlay of the FFI-42 tests: each entry asserts the operator method it names safe to call
# ([FFI-2]), which is what lets it implement its Ember operator interface. `sub` has no entry.

unsafe overlay cpp "operators.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn add(other)
    fn neg()
    fn add_assign(other)
    fn eq(other)
    fn mul(k)
    fn index(i)
    fn index_mut(i)
    fn total()
