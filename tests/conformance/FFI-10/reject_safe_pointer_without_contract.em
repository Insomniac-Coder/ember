#$ test: compile-fail
#$ rules: FFI-10, FFI-1, FFI-2
#$ profiles: debug
#$ error[E5002]: `safe fn` needs an `@ffi` contract for pointer parameter `p`

unsafe extern "C":
    safe fn inspect(p: *u8) -> i32

fn main():
    pass
