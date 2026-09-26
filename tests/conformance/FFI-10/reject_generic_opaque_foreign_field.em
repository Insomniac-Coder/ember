#$ test: compile-fail
#$ rules: FFI-10, FFI-8
#$ profiles: debug
#$ error[E5050]: `Handle` is incomplete and cannot cross the C boundary by value

unsafe extern "C":
    type Handle

struct Holder[T]:
    value: T

fn main():
    value: Holder[Handle]
