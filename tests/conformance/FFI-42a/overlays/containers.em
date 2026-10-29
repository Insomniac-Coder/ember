# The overlay of the FFI-42a tests (a copy of FFI-42's): each entry asserts the method it names
# safe to call ([FFI-2]); `iter` is the one a `for` loop calls.

unsafe overlay cpp "containers.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Bag:
        fn new()
        fn put(value)
        fn size()
        fn iter()
    class Row:
        fn iter()
