#$ test: compile-fail
#$ rules: FFI-11

unsafe extern "C":
    @ffi(param(input, borrowed, count(n)), result(borrowed, count(n), exclusive, from(input)))
    safe fn wrong(input: Span[i32], n: usize) -> Span[i32] #$ error[E5002]: `@ffi` exclusive counted result needs `MutSpan[T]`

fn main():
    pass
