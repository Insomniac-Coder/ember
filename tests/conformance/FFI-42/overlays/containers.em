# The overlay of the FFI-42 and FFI-42a iteration tests: each entry asserts the method it names
# safe to call ([FFI-2]); `iter` is the one a `for` loop calls.

unsafe overlay cpp "containers.hpp":
    fn new()
    fn put(value)
    fn size()
    fn iter()
