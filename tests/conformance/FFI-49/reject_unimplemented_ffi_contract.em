#$ test: compile-fail
#$ rules: FFI-11, FFI-1, ATT-6
#$ profiles: debug
# `[FFI-11]` (D-746) — an `@ffi` argument on a foreign function is a form of the contract
# vocabulary: `ownership="transfer"` is none (`E0104`, naming what `@ffi` takes); `handle` is a word
# of a type's or a result's contract, not a function's (`[FFI-1]`, ODR-247): on a function it is a
# contract error (`E5002`), whose help names the type form (`@ffi(handle) type T`).

unsafe extern "C":
    @ffi(ownership="transfer")    #$ error[E0104]: `ownership` is not a contract word of `@ffi`
    fn acquire() -> i32
    @ffi(handle)    #$ error[E5002]: `handle` is a word of a type's or a result's contract, not a function's
    fn on_main() -> i32
    @ffi(param(p, borrowed, one, forever))    #$ error[E0104]: `forever` is not a contract word of `@ffi`
    fn first(p: *i32) -> i32

fn main():
    pass
