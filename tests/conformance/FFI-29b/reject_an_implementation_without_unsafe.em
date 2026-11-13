#$ test: compile-fail
#$ rules: FFI-29b, FFI-51
# `[FFI-29b]` — a single-header library's implementation is compiled in an `unsafe import c`
# (`[FFI-51]`): it is C compiled into the program, and without `unsafe` the import is `E3100`.

import c "single.h" with (implementation = ["SINGLE_IMPLEMENTATION"])    #$ error[E3100]: `import c "single.h"` compiles `single.h`'s implementation (`SINGLE_IMPLEMENTATION`) into the program, where its code runs unchecked, before `main` too; to vouch for it, write `unsafe import c "single.h" with (implementation = ["SINGLE_IMPLEMENTATION"])` with a `# SAFETY:` note above it

fn main():
    pass
