#$ test: run-pass
#$ rules: FFI-51
#$ stdout: 6 3 4
# `[FFI-51]` — a header that only declares (types, macros, declarations of functions, and `static
# inline` functions, compiled only where they are called) needs no `unsafe`: its functions run only
# when they are called, and those calls are fenced as any C call is.

import c "declares.h" as d

fn main():
    p = d.Pair(a=3, b=4)
    println(d.DECLARES_SIX, p.a, p.b)
