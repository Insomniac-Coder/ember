#$ test: compile-fail
#$ rules: FFI-1, FFI-11, FFI-33b
#$ profiles: debug
# `[FFI-1]`, `[FFI-11]` (ODR-247) — `threads(…)` is `any`, `main` or `creator`, and anything else is
# `E0104`. `threads(creator)` and `handle` are words of a type's or a result's contract, not a
# function's: a function has no creator of its own (the word binds an object, a callback or a
# `retained` parameter to the thread that made it) and is not a handle, so on a function each is a
# contract error, `E5002`, whose help says where the word goes. On a type both are accepted.

unsafe extern "C":
    @ffi(threads(creator))    #$ error[E5002]: `threads(creator)` is a word of a type's or a result's contract, not a function's
    fn made() -> i32
    @ffi(handle, link_name="made")    #$ error[E5002]: `handle` is a word of a type's or a result's contract, not a function's
    fn handled() -> i32
    @ffi(threads(sometimes))    #$ error[E0104]: `threads(…)` is `any`, `main` or `creator`
    fn sometimes() -> i32
    @ffi(threads(any), threads(main))    #$ error[E0104]: duplicate `threads(…)` in `@ffi`
    fn twice() -> i32
    @ffi(handle, threads(creator), destroys(release))
    type Widget
    fn release(w: *mut Widget)

fn main():
    pass
