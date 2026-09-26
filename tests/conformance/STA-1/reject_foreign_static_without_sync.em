#$ test: compile-fail
#$ rules: STA-1, FFI-10
#$ error[E7002]: foreign static type `*i32` is not Sync

unsafe extern "C":
    static foreign_pointer: *i32

fn main():
    pass
