#$ test: compile-fail
#$ rules: FFI-11, FFI-10
#$ profiles: debug
#$ help: write the declaration `unsafe fn raw_abs(…)`, with `@safety("…")` above it
# `[FFI-11]`, `[FFI-10]` (verdict 133) — `unsafe` is no word of `@ffi(…)`: a hand-written declaration
# unsafe to call is `@safety("…")` above `unsafe fn`, as an overlay entry is. `@ffi(unsafe)` and its
# forms for one slot, `param(p, unsafe)` and `result(unsafe)`, are `E0104`, whose message gives the
# line to write; `@ffi(…)` keeps its other words.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(unsafe, link_name="abs")    #$ error[E0104]: `unsafe` is no contract word
    @safety("callers pass what the C function `raw_abs` requires")
    fn raw_abs(x: i32) -> i32
    @ffi(param(p, unsafe))    #$ error[E0104]: `unsafe` is no contract word
    @safety("`p` must point at a live i32")
    fn peek(p: *i32) -> i32
    @ffi(result(unsafe))    #$ error[E0104]: `unsafe` is no contract word
    @safety("the pointer it returns may be null")
    fn first() -> *i32

fn main():
    unsafe:  # SAFETY(ffi): `abs` reads its argument only
        println(raw_abs(-5))
