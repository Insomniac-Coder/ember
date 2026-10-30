#$ test: compile-fail
#$ rules: FFI-10, FFI-2, UNS-7
#$ profiles: debug
# `[FFI-10]` (verdict 133) — a hand-written C declaration unsafe to call is `@safety("…")` above
# `unsafe fn`, as an overlay entry is: its calls are left to unsafe code on purpose, so a call outside
# it gets the error any `unsafe fn` call gets, which asks for no fact. `@ffi(…)` keeps its other words
# (`link_name`). `safe` and `unsafe` together on one declaration is `E5002`.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(link_name="abs")
    @safety("callers pass what the C function `raw_abs` requires")
    unsafe fn raw_abs(x: i32) -> i32
    @safety("`p` must point at a live i32")
    unsafe fn peek(p: *i32) -> i32
    @ffi(link_name="labs")
    @safety("callers pass what the C function `both` requires")
    safe unsafe fn both(x: i64) -> i64    #$ error[E5002]: `both` is declared both `safe` and `unsafe`

fn main():
    println(raw_abs(-3))    #$ error[E3100]: calls an `unsafe fn`, whose callers must make true what its `@safety` says
    value: i32 = 4
    println(peek(ref_to_ptr(ref value)))    #$ error[E3100]: calls an `unsafe fn`, whose callers must make true what its `@safety` says
    unsafe:  # SAFETY(ffi): `abs` reads its argument only, and `value` is a live i32
        println(raw_abs(-5), peek(ref_to_ptr(ref value)))
