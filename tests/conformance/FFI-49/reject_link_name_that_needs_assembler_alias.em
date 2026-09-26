#$ test: compile-fail
#$ rules: FFI-49
#$ profiles: debug
#$ error[E0900]: a `link_name` that is not a C identifier is not implemented yet

unsafe extern "C":
    @ffi(link_name="_abs@4")
    fn magnitude(x: i32) -> i32

fn main():
    pass
