# The overlay of the FFI-40 tests: every member has an entry, `total` asserted as only reading its
# object; `parent`'s entry gives its `Node&` result no lifetime word.

unsafe overlay cpp "shelf.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Cache:
        fn new()
        fn count(s)
        fn grow(s)
        fn add(s)
        @ffi(const)
        fn total()
        fn clear()
    class Node:
        fn new()
        fn parent()
