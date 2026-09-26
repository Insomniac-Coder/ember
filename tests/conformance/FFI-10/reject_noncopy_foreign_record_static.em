#$ test: compile-fail
#$ rules: FFI-10, FFI-8
#$ error[E0900]: foreign static access requires a Copy C-compatible value without borrowed fields

struct ForeignPair:
    left: i32
    right: i32

unsafe extern "C":
    static foreign_pair: ForeignPair

fn main():
    pass
