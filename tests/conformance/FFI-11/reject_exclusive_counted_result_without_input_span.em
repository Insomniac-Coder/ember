#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ error[E5002]: result `count(n)` needs an input span sharing `n`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(result(borrowed, count(n), exclusive, from(static)))
    safe fn no_length(n: usize) -> MutSpan[i32]

fn main():
    pass
