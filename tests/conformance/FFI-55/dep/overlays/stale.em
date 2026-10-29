# Another package's overlay of `lib.h`, checked against another version of the header: its package's
# `ember.overlays` records `lib_twice` as that version declared it.

unsafe overlay c "lib.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn lib_twice(x)
