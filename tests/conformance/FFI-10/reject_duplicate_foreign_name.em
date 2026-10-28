#$ test: compile-fail
#$ rules: FFI-10, TYP-26
#$ profiles: debug
#$ error[E1030]: `abs` is already declared in this module

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    safe fn abs(x: i32) -> i32

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    safe fn abs(x: i32) -> i32

fn main():
    pass
