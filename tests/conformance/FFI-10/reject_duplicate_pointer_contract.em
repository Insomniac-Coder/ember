#$ test: compile-fail
#$ rules: FFI-10, FFI-11
#$ profiles: debug
#$ error[E0104]: duplicate pointer contract in `@ffi`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(exponent, borrowed, one, exclusive), param(exponent, borrowed, one, exclusive))
    safe fn frexp(x: f64, mut exponent: i32) -> f64

fn main():
    pass
