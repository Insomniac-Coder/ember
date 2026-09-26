#$ test: compile-fail
#$ rules: FFI-11

unsafe extern "C":
    @ffi(param(input, borrowed, count(n)), result(borrowed, count(n), nullable, exclusive, from(input)))
    safe fn wrong(input: Span[i32], n: usize) -> Option[Span[i32]] #$ error[E5002]: `@ffi` nullable exclusive counted result needs `Option[MutSpan[T]]`

fn main():
    pass
