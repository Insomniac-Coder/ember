#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `safe fn` needs an `@ffi` contract for reference parameter `value`

unsafe extern "C":
    safe fn read_one(value: ref i32) -> i32

fn main():
    pass
