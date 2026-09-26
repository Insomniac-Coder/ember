#$ test: run-pass
#$ rules: FFI-10, FFI-11, FFI-49
#$ profiles: debug, release, shipping
#$ stdout: 6

unsafe extern "C":
    @ffi(param(exponent, borrowed, one, exclusive), link_name="frexp")
    safe fn split(x: f64, mut exponent: i32) -> f64

fn main():
    exponent: i32 = 0
    split(42.0, exponent)
    println(exponent)
