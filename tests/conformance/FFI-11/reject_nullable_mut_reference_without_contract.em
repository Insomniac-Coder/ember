#$ test: compile-fail
#$ rules: FFI-11, FFI-10
#$ profiles: debug
#$ error[E5002]: `safe fn` needs an `@ffi` contract for nullable mutable reference parameter `value`

unsafe extern "C":
    safe fn update_optional(value: Option[ref mut i32]) -> i32

fn main():
    pass
