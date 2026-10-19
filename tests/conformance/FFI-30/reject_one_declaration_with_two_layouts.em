#$ test: compile-fail
#$ rules: FFI-30
#$ error[E5011]: `Vec2` is imported from `common.h` with a layout other than the one `common.h` gave it
# `[FFI-30]` — one C declaration imported with two layouts in one build is `E5011`: `helpers.narrow`
# imports `common.h` as written, and this file with `COMMON_WIDE`, which adds a field, both binding
# `c` (the import loaded second is the one reported).

import helpers.narrow
import c "common.h" with (defines = ["COMMON_WIDE"])

fn main():
    println(narrow.length2(1.0, 2.0))
