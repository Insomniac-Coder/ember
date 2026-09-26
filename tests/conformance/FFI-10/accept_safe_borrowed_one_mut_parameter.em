#$ test: run-pass
#$ rules: FFI-10, FFI-11
#$ profiles: debug, release, shipping
#$ stdout: 6

unsafe extern "C":
    @ffi(param(exponent, borrowed, one, exclusive))
    safe fn frexp(x: f64, mut exponent: i32) -> f64

fn main():
    exponent: i32 = 0
    frexp(42.0, exponent)
    println(exponent)
