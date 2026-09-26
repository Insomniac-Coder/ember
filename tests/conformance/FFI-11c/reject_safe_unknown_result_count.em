#$ test: compile-fail
#$ rules: FFI-11c, FFI-10
#$ profiles: debug
#$ error[E5002]: `safe fn` has an unknown count for its result

unsafe extern "C":
    @ffi(result(borrowed, TODO(count), from(static)))
    safe fn inspect() -> ref i32

fn main():
    pass
