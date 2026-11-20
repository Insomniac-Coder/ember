# The overlay of the FFI-42a test: `first` is asserted as only reading its object; `bump`, and the
# `const` `looked`, whose entry does not say so, change it ([FFI-40a], verdict 132).

unsafe overlay cpp "rows.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Row:
        fn new()
        fn iter()
        @ffi(const)
        fn first()
        fn bump()
        fn looked()
