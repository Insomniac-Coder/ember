#$ test: run-pass
#$ rules: FFI-30a
#$ profiles: debug, development, release
#$ stdout: 7 7
# `[FFI-30a]` — an overlay changes the signatures, safety and names of functions; it never changes
# a type's identity, layout or fields. Two imports of `pair.h` with different overlays (one names
# the function `total`, the other `add_up`) share one `Pair`: a value made through one is passed
# to the other's function.

import c "pair.h" with (overlay = "overlays/plain.em") as first
import c "pair.h" with (overlay = "overlays/other.em") as second

fn main():
    p = first.Pair(left=3, right=4)
    println(first.total(p), second.add_up(p))
