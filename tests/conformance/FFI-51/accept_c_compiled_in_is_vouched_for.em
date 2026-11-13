#$ test: run-pass
#$ rules: FFI-51, BLD-FFI-1
#$ warning[L3015]: `bridge_add` is a C function the program calls in unsafe code
#$ stdout: 42
# `[FFI-51]` — C compiled into the program runs code nothing checks, so the import that compiles a C
# source file is written `unsafe import c`, with a `# SAFETY:` note above it saying why that code is
# sound. `[BLD-FFI-1]` — it compiles `bridge.c` and imports `bridge.h`.

# SAFETY: bridge.c is ours: it defines `bridge_add` and runs nothing before `main`
unsafe import c "./bridge.c" as b

fn main():
    # SAFETY(ffi): a sum of two numbers
    unsafe:
        println(b.bridge_add(40, 2))
