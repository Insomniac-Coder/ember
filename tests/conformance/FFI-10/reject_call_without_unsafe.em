#$ test: compile-fail
#$ rules: FFI-10, FFI-2
#$ profiles: debug
#$ error[E3100]: `abs(-42)` (line 16) calls an `unsafe fn`, whose callers must make true what its `@safety` says
#$ error[E5002]: `labs` does not say whether it is safe to call
# `[FFI-10]` (verdict 136) — a hand-written C declaration says whether it is safe to call: an `unsafe fn`'s
# call outside unsafe code is `E3100`, and a plain `fn`, safe to call everywhere else in Ember, is `E5002`.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `abs` requires")
    unsafe fn abs(x: i32) -> i32
    @safety("callers pass what the C function `labs` requires")
    fn labs(x: i64) -> i64

fn main():
    println(abs(-42))
