#$ test: compile-fail
#$ rules: FFI-1, FFI-11, FFI-33b
#$ profiles: debug
# `[FFI-11]` — `threads(…)` is `any`, `main` or `creator`; a function has no creator of its own
# (the word binds a callback, a `retained` parameter or a `returns_owned` result), so on a
# function it is not built (`E0900`), and anything else is `E0104`.

unsafe extern "C":
    @ffi(threads(creator))    #$ error[E0900]: `threads(creator)` on a function is not implemented yet
    fn made() -> i32
    @ffi(threads(sometimes))    #$ error[E0104]: `threads(…)` is `any`, `main` or `creator`
    fn sometimes() -> i32
    @ffi(threads(any), threads(main))    #$ error[E0104]: duplicate `threads(…)` in `@ffi`
    fn twice() -> i32

fn main():
    pass
