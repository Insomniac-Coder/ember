#$ test: compile-fail
#$ rules: FFI-11, FFI-11c, DIA-18
#$ profiles: debug
#$ not-help: state the facts on its declaration
# `[FFI-11]` — the contract word `unsafe` leaves a call unsafe on purpose, for the whole function
# (`@ffi(unsafe)`) or one slot (`param(p, unsafe)`, `result(unsafe)`): its call outside `unsafe:` is
# `E5002`, which then names the word rather than asking for facts. A `safe fn` with it is `E5002`
# too.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(unsafe, link_name="abs")
    fn raw_abs(x: i32) -> i32
    @ffi(param(p, unsafe))
    fn peek(p: *i32) -> i32
    @ffi(result(unsafe))
    fn first() -> *i32
    @ffi(unsafe, link_name="labs")    #$ error[E5002]: `unsafe` leaves the call unsafe
    safe fn safe_abs(x: i32) -> i32

fn main():
    println(raw_abs(-3))    #$ error[E5002]: contract leaves its calls `unsafe` on purpose
    value: i32 = 4
    println(peek(ref_to_ptr(ref value)))    #$ error[E5002]: contract leaves its calls `unsafe` on purpose
    pointer = first()    #$ error[E5002]: contract leaves its calls `unsafe` on purpose
    unsafe:  # SAFETY(ffi): `abs` reads its argument only
        println(raw_abs(-5))
