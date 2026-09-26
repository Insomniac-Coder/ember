#$ test: compile-fail
#$ rules: FFI-10, FFI-8
#$ profiles: debug
#$ error[E5050]: `[Handle; 2]` is incomplete and has no size or alignment

unsafe extern "C":
    type Handle

type Two = [Handle; 2]

fn main():
    println(size_of[Two]())
