#$ test: compile-fail
#$ rules: FFI-10, FFI-8
#$ profiles: debug
#$ error[E5015]: `[Handle; 2]` is incomplete and has no size or alignment

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    type Handle

type Two = [Handle; 2]

fn main():
    println(size_of[Two]())
