#$ test: compile-fail
#$ rules: FFI-10
#$ error[E0900]: foreign static reads currently require a C scalar type

unsafe extern "C":
    static text: str

fn main():
    pass
