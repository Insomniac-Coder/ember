#$ test: compile-fail
#$ rules: FFI-49, ATT-6
#$ profiles: debug
#$ error[E0900]: only `@ffi` link names and borrowed-one pointer contracts are implemented yet

unsafe extern "C":
    @ffi(ownership="transfer")
    fn acquire() -> i32

fn main():
    pass
