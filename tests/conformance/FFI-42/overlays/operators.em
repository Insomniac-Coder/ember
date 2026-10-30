# The overlay of the FFI-42 tests: each entry asserts the operator method it names safe to call
# ([FFI-2]), which is what lets it implement its Ember operator interface. `sub` has no entry.

unsafe overlay cpp "operators.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Money:
        @ffi(const)
        fn add(b)
        fn add_assign(b)
        @ffi(const)
        fn eq(b)
    class Vec2:
        @ffi(const)
        fn add(other)
        @ffi(const)
        fn neg()
        fn add_assign(other)
        @ffi(const)
        fn eq(other)
        @ffi(const)
        fn mul(k)
    class Points:
        @ffi(const)
        fn mul(k)
    class Grid:
        @ffi(const)
        fn index(i)
        fn index_mut(i)
        @ffi(const)
        fn total()
