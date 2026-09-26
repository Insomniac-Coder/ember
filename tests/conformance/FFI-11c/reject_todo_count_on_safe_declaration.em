#$ test: compile-fail
#$ rules: FFI-11c, FFI-10
#$ profiles: debug
#$ error[E5002]: `safe fn` has an unknown count for parameter `data`

unsafe extern "C":
    @ffi(param(data, borrowed, TODO(count)))
    safe fn inspect(mut data: i32) -> i32

fn main():
    pass
