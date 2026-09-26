#$ test: compile-fail
#$ rules: FFI-10, FFI-8
#$ profiles: debug
#$ error[E5050]: `Handle` is incomplete and has no size or alignment

unsafe extern "C":
    type Handle

fn main():
    println(size_of[Handle]())
