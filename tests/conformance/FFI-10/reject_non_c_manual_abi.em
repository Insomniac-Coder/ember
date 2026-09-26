#$ test: compile-fail
#$ rules: FFI-10, FFI-9
#$ profiles: debug
#$ error[E0900]: only `unsafe extern "C"` declaration blocks are implemented yet

unsafe extern "vectorcall":
    safe fn abs(x: i32) -> i32

fn main():
    pass
