#$ test: compile-fail
#$ rules: FFI-11, FFI-1, ATT-6
#$ profiles: debug
# `[FFI-11]` (D-746) — an `@ffi` argument on a foreign function is a form of the contract
# vocabulary: `ownership="transfer"` is none (`E0104`, naming what `@ffi` takes); `handle` is a word
# of a type's contract, not a function's (`[FFI-1]`, ODR-247, ODR-269): on a function it is a
# contract error (`E5002`), whose help names the type form (`@ffi(handle) type T`).

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(ownership="transfer")    #$ error[E0104]: `ownership` is not a contract word of `@ffi`
    @safety("callers pass what the C function `acquire` requires")
    unsafe fn acquire() -> i32
    @ffi(handle)    #$ error[E5002]: `handle` is a word of a type's contract, not a function's
    @safety("callers pass what the C function `on_main` requires")
    unsafe fn on_main() -> i32
    @ffi(param(p, borrowed, one, forever))    #$ error[E0104]: `forever` is not a contract word of `@ffi`
    @safety("callers pass what the C function `first` requires")
    unsafe fn first(p: *i32) -> i32

fn main():
    pass
