#$ test: compile-fail
#$ rules: FFI-25
#$ profiles: debug
#$ error[E0900]: only `on_panic=abort` is implemented for exported functions

@export(on_panic=unwind)
pub fn unwind_not_supported() -> i32:
    return 0

fn main():
    pass
