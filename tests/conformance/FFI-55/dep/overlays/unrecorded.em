# Another package's overlay of `lib.h`, which its package's `ember.overlays` does not record.

unsafe overlay c "lib.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn lib_twice(x)
