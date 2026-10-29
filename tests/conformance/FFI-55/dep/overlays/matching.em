# Another package's overlay of `lib.h`, checked against the header these tests import: its package's
# `ember.overlays` records that.

unsafe overlay c "lib.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn lib_twice(x)
