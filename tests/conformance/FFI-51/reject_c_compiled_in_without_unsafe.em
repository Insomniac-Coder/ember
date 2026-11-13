#$ test: compile-fail
#$ rules: FFI-51
# `[FFI-51]` — without `unsafe`, an import that compiles a C source file into the program is `E3100`,
# whose message gives the import to write.

import c "./bridge.c" as b    #$ error[E3100]: `import c "./bridge.c"` compiles `bridge.c` into the program, where its code runs unchecked, before `main` too; to vouch for it, write `unsafe import c "./bridge.c" as b` with a `# SAFETY:` note above it

fn main():
    pass
