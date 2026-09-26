#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: result `count(n)` needs an input span sharing `n`

unsafe extern "C":
    @ffi(result(borrowed, count(n), from(static)))
    safe fn no_length(n: usize) -> Span[i32]

fn main():
    pass
