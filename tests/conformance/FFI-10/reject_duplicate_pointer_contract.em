#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E0104]: duplicate pointer contract in `@ffi`

unsafe extern "C":
    @ffi(param(exponent, borrowed, one, exclusive), param(exponent, borrowed, one, exclusive))
    safe fn frexp(x: f64, mut exponent: i32) -> f64

fn main():
    pass
