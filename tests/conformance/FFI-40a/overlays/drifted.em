# The overlay of ../reject_a_method_line_that_differs_from_the_header.em: method lines that do not
# match the header's methods.

unsafe overlay cpp "cache.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Cache:
        fn lookups(self) -> i64
        fn add(self, amount: c_int)
        fn missing(self)
        fn hits() -> c_int
    class Nothing:
        fn lookups(self) -> c_int
