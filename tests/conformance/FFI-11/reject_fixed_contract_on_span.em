#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `@ffi` contract for `data` needs a pointer carrier
#$ error[E5002]: `safe fn` needs an `@ffi` count contract for span parameter `data`

unsafe extern "C":
    @ffi(param(data, borrowed, fixed(2)))
    safe fn wrong_carrier(data: Span[i32]) -> i32

fn main():
    pass
