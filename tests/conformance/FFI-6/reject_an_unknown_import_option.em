#$ test: compile-fail
#$ rules: FFI-6
#$ help: the options are `include_paths`, `defines`, `link`, `overlay` and `implementation`
# `[FFI-6]` — `import c` takes `include_paths`, `defines`, `link`, `overlay` and `implementation`;
# an unknown option is `E0104`, as is an option written in another shape.

import c "inline_math.h" with (packing = 4) as a  #$ error[E0104]: `packing` is not an option of `import c`
import c "inline_math.h" with (defines = "INLINE_MATH_SCALE=2") as b  #$ error[E0104]: `defines` is a list of strings
import c "inline_math.h" with (overlay = [1]) as d  #$ error[E0104]: `overlay` names an overlay file, or a list of them

fn main():
    pass
