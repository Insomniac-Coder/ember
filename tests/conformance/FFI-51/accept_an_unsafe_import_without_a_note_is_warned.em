#$ test: run-pass
#$ rules: FFI-51, UNS-8, LEX-23
#$ warning[L3015]: `bridge_add` is a C function the program calls in unsafe code
#$ stdout: 5
# `[FFI-51]` — an `unsafe import c` without a `# SAFETY:` note on the line above it is `W3012`; the
# program still builds.

unsafe import c "./bridge.c" as b    #$ warning[W3012]: this `unsafe` has no safety note

fn main():
    # SAFETY(ffi): a sum of two numbers
    unsafe:
        println(b.bridge_add(2, 3))
