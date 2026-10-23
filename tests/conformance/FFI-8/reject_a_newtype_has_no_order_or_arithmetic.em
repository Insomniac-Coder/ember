#$ test: compile-fail
#$ rules: FFI-8, TYP-36
# ODR-270 — a newtype compares with `==` and `!=` as its representation does, and has no order
# (`Ord`) and no arithmetic: `(a as c_int) < (b as c_int)` says it of the representations.

from std.ffi import c_int

@ffi(newtype)
type fd_t = c_int

fn main():
    a = 7 as fd_t
    c = 9 as fd_t
    println(a < c)  #$ error[E2040]: `fd_t` does not implement `Ord`
    d = a + c  #$ error[E2020]: `+` cannot be applied to `fd_t`
    println((a as c_int) < (c as c_int))
