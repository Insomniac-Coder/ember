#$ test: compile-fail
#$ rules: FFI-49, ATT-6
#$ profiles: debug
#$ error[E0900]: only `@ffi(link_name="C_identifier")` is implemented yet

unsafe extern "C":
    @ffi(ownership="transfer")
    fn acquire() -> i32

fn main():
    pass
