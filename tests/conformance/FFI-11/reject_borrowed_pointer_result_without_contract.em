#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `safe fn` needs an `@ffi` contract for reference result

unsafe extern "C":
    safe fn static_number() -> ref i32

fn main():
    pass
