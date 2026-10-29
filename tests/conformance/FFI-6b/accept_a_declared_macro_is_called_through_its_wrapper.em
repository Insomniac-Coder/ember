#$ test: run-pass
#$ rules: FFI-6b, FFI-29
#$ warning[L3015]: `PAIR_FIRST` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ warning[W5001]: macro `HIDDEN_ID` is not imported from `macros.h`: a function-like macro
#$ warning[W5001]: macro `LOG_ALL` is not imported from `macros.h`: a function-like macro
#$ warning[W5001]: macro `twice` is not imported from `macros.h`: a function-like macro
#$ stdout: 25 4.5 7 7 true false
# `[FFI-6b]` (ODR-147) — a function-like macro is exposed when an overlay entry marked
# `@ffi(macro_fn)` declares its signature, in Ember types since a macro has none in C; `[FFI-29]` —
# the import's shim holds a C wrapper for each such macro, which its binding calls. Scalars and the
# header's own structs cross by value; a raw pointer leaves the call `unsafe`. A macro no entry
# declares is still `W5001`.

import c "macros.h" with (overlay = "overlays/macros.em")

fn main():
    point = c.Point(x = 3, y = 4)
    pair = c.Pair(first = 7, second = 8)
    first: i64 = 0
    unsafe:  # SAFETY(ffi): `pair` is alive for the call, which only reads it
        first = c.PAIR_FIRST((ref pair) as *c.Pair)
    println(c.SQUARE(5), c.SCALE(1.5, 3.0), c.POINT_SUM(point), first, c.IS_EVEN(6), c.IS_EVEN(7))
