#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E5002]: `safe fn` needs an `@ffi` contract for mutable parameter `exponent`

unsafe extern "C":
    safe fn frexp(x: f64, mut exponent: i32) -> f64

fn main():
    pass
