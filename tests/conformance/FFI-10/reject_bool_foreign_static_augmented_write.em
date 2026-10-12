#$ test: compile-fail
#$ rules: FFI-10, TYP-21
#$ error[E2020]: `+=` is not defined on `bool`

unsafe extern "C":
    static mut foreign_flag: bool

fn main():
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        foreign_flag += true
