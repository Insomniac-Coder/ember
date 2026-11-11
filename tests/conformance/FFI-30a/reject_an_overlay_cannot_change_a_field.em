#$ test: compile-fail
#$ rules: FFI-30a
#$ error[E0100]: expected an overlay item
#$ profiles: debug
# `[FFI-30a]` — an overlay never changes a type's layout or fields: this one tries to give `left`
# another type.

import c "pair.h" with (overlay = "overlays/retype.em") as p

fn main():
    x = p.Pair(left=1, right=2)
    println(x.left)
