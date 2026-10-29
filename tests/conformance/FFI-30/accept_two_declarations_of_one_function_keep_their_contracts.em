#$ test: run-pass
#$ rules: FFI-30, FFI-30a
#$ warning[L3015]: `pointers_read` is a C function the program calls in unsafe code
#$ warning[L3015]: `pointers_bump` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 4 4 5 6
# `[FFI-30a]` — an overlay changes a function's signature and safety: `pointers.h` imported with an
# overlay takes a borrow of an `i32` and is safe to call; imported again without one, the same C
# functions take raw pointers and need `unsafe`. Each import's declaration keeps its own contract,
# and both call the one C function (`[FFI-30]`): C sees one prototype of each, whatever Ember's
# spellings of the two.

import c "pointers.h" with (implementation = ["POINTERS_IMPLEMENTATION"], overlay = "overlays/pointers.em")
import c "pointers.h" with (defines = ["POINTERS_RAW"]) as raw

fn main():
    x: i32 = 4
    read = c.pointers_read(x)
    through_raw: i32 = 0
    bumped: i32 = 0
    unsafe:  # SAFETY(ffi): `x` is alive for both calls, which read and write one `int`
        through_raw = raw.pointers_read((ref x) as *i32)
        bumped = raw.pointers_bump((ref mut x) as *mut i32)
    c.pointers_bump(x)
    println(read, through_raw, bumped, x)
