#$ test: run-pass
#$ rules: FFI-12, FFI-2
#$ profiles: debug, development, release
#$ stdout: 10 12
# `[FFI-12]` — an overlay entry names a function of the header, with its parameters, in order, by
# their names in the header: this one matches, and its calls are safe (`[FFI-2]`).

import c "tally.h" with (overlay = "tally_overlay.em")

fn main():
    items: Array[i32] = [1, 2, 3, 4]
    println(c.tally_sum(items.as_span()), c.tally_scale(3, 4))
