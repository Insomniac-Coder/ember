#$ test: compile-fail
#$ rules: FFI-10, FFI-49
#$ error[E5002]: foreign static aliases for one C symbol must have the same type and const contract

unsafe extern "C":
    @ffi(link_name="one_global")
    static first: i32
    @ffi(link_name="one_global", immutable)
    static second: i32

fn main():
    pass
