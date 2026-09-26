#$ test: compile-fail
#$ rules: FFI-11

unsafe extern "C":
    @ffi(param(data, borrowed, count(n)))
    safe fn wrong(data: Option[Span[i32]], n: usize) -> i32 #$ error[E5002]: `@ffi` contract for `data` needs a pointer carrier

fn main():
    pass
