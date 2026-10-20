#$ test: run-pass
#$ rules: FFI-5, FFI-8
#$ profiles: debug, development, release
#$ stdout: 7 7 | 16 16 | 24 16
# `[FFI-5]` — the importer records each struct's size, alignment and field offsets from Clang for
# this target, and the compiler checks its own layout against them. `[FFI-8]` — `#pragma pack` is
# `@packed`, `_Alignas` `@align(N)`: Ember's `size_of`/`align_of` equal C's, which the header's
# own functions report.

import c "layouts.h"
from std.mem import size_of, align_of

fn main():
    unsafe:  # SAFETY(ffi): scalar calls
        println(size_of[c.Wire](), c.layouts_wire_size(), "|", align_of[c.Lane](), c.layouts_lane_align(), "|", size_of[c.Mixed](), c.layouts_mixed_c())
