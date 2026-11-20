# Clause FFI-22.2: "and it changes the caller's real variables only by their own operations, as an
# `Atomic`, a lock or a `guarded by` field under its lock is changed (`E7015`"
#$ test: compile-fail
#$ rules: FFI-22, FFI-21
# The spec's message, with the overlay it names (`overlays/visits.em`, no thread word). `visits.h`
# only declares, so its import needs no `unsafe` (`[FFI-51]`); the test only compiles.

from std.ffi import c_int
import c "visits.h" with (overlay = "overlays/visits.em") as v

fn main():
    seen: Array[c_int] = []
    v.visit_each(3, fn(i: c_int): seen.push(owns i))    #$ error[E7015]: C may call `visit` from any thread at once (`visit_each`'s `visit` has no `threads(...)` word), and it pushes to `seen`, a list of this thread; count with an atomic, or, if C calls it only on this thread, say so in the overlay: `visit: callback(borrowed, user_data=data) threads(creator)`
    println(seen.len())
