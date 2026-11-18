#$ test: run-pass
#$ rules: FFI-11, FFI-52
#$ stdout: 5 1.0
# `[FFI-11]` — on a function, `no_callbacks` says it never calls back into Ember (`[FFI-52]`), and it
# shares one `@ffi(…)` with the other words, `effects=[…]` here.

from std.ffi import c_int, c_double

unsafe extern "C":  # SAFETY(ffi): the C library's own `abs` and `cos`
    @ffi(effects=[], no_callbacks)
    safe fn abs(x: c_int) -> c_int
    @ffi(effects=[], no_callbacks)
    safe fn cos(x: c_double) -> c_double

fn main():
    println(abs(-5), cos(0.0))
