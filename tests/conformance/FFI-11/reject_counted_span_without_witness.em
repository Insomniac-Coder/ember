#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `count(n)` needs integer ABI witness `n`

unsafe extern "C":
    @ffi(param(data, borrowed, count(n)))
    safe fn inspect(data: Span[u8]) -> i32

fn main():
    pass
