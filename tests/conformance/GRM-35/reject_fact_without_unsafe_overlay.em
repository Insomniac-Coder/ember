#$ test: compile-fail
#$ rules: GRM-35, TIER-1, FFI-2
# `[GRM-35]` — an overlay that states any fact about foreign code — a function
# entry (which asserts the function safe to call), a contract word, an
# `@ffi(…)` attribute, an `@ffi(macro_fn)` entry (ODR-147) — is an assertion,
# and is declared `unsafe overlay` (`E5067`), the third boundary of `[TIER-1]`.

overlay c "math.h":    #$ error[E5067]: is not declared `unsafe overlay`
    fn cos(x)

overlay c "zlib.h":    #$ error[E5067]: is not declared `unsafe overlay`
    rename deflateInit_ as deflate_init
    @ffi(handle)
    type z_stream

overlay c "string.h":    #$ error[E5067]: is not declared `unsafe overlay`
    fn strlen(s: borrowed nul_terminated)

overlay c "macros.h":    #$ error[E5067]: is not declared `unsafe overlay`
    @ffi(macro_fn)
    fn SQUARE(x: i32) -> i32

fn main():
    pass
