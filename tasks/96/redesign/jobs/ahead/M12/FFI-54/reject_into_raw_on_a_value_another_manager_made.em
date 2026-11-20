# Clause FFI-54.1: "It is written on a value whose block the runtime laid out or C gave (...), which
# `ember_rt_object_release` can end; on a value Ember made `using` another manager it is `E2114`"
#$ test: compile-fail
#$ rules: FFI-54, OWN-12, TYP-46, ALC-1
# The message's first part, which names `w` and its arena; its help writes the value made again as
# `second = owns Widget(id=2)`, a name and a type the message's own program does not show (Q3), so
# the claim stops before it. The test only compiles.

struct Point:
    x: int

fn main():
    frame = Arena.with_capacity(64)
    w = owns Point(x=2) using frame
    # SAFETY: the test only compiles; nothing is given to C
    unsafe:
        _p = w.into_raw()    #$ error[E2114]: `w` was made in `frame`, an `Arena`, and C would end it through the runtime, which gives back only blocks of the built-in manager
