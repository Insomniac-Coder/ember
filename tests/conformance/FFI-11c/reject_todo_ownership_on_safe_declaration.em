#$ test: compile-fail
#$ rules: FFI-11c, FFI-10
#$ profiles: debug
#$ error[E5002]: `safe fn` has an unknown ownership for parameter `data`

unsafe extern "C":
    @ffi(param(data, TODO(ownership), one))
    safe fn inspect(mut data: i32) -> i32

fn main():
    pass
