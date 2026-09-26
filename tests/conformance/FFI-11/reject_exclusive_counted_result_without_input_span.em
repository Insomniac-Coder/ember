#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ error[E5002]: result `count(n)` needs an input span sharing `n`

unsafe extern "C":
    @ffi(result(borrowed, count(n), exclusive, from(static)))
    safe fn no_length(n: usize) -> MutSpan[i32]

fn main():
    pass
