#$ test: compile-fail
#$ rules: FFI-11, ATT-6
#$ profiles: debug
# `[FFI-11]` (D-746) — an `@ffi` argument on a foreign function is a form of the contract
# vocabulary: `ownership="transfer"` is none (`E0104`, naming what `@ffi` takes); `threads(main)` is
# one this compiler has not built (`E0900`).

unsafe extern "C":
    @ffi(ownership="transfer")    #$ error[E0104]: `ownership` is not a contract word of `@ffi`
    fn acquire() -> i32
    @ffi(threads(main))    #$ error[E0900]: this `threads(…)` contract is not implemented yet
    fn on_main() -> i32
    @ffi(param(p, borrowed, one, forever))    #$ error[E0104]: `forever` is not a contract word of `@ffi`
    fn first(p: *i32) -> i32

fn main():
    pass
