#$ test: compile-fail
#$ rules: FFI-8
# `[FFI-8]` — `c_longdouble` is opaque bytes: no arithmetic, no comparison, no conversion.

from std.ffi import c_longdouble

fn sum(a: c_longdouble, b: c_longdouble) -> c_longdouble:
    return a + b    #$ error[E2020]

fn widen(x: f64) -> c_longdouble:
    return x as c_longdouble    #$ error[E2020]

fn main():
    pass
