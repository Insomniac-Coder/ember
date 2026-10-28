# Another package's overlay of `lib.h` that records no hash.

unsafe overlay c "lib.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn lib_twice(x)
