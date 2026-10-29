#$ test: compile-fail
#$ rules: FFI-8
#$ warning[L3015]: `sum_wide` is a C function the program calls in unsafe code
# `[FFI-8]` — a variadic C function is called only in `unsafe` (`E5002`); each further argument has
# the type the program gives it, one C passes unchanged (`E5015`: an untyped literal; `E5013`: a type
# C's default argument promotions change, naming only the promotion); the function is called, never a
# value.

from std.ffi import c_int
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(variadic, param(format, borrowed, nul_terminated))
    @safety("the format reads what each call passes")
    fn printf(format: cstr) -> c_int

fn main():
    narrow: i16 = 3
    single: f32 = 1.5
    flag: bool = true
    letter: char = 'a'
    total = v.sum_wide(1, 5i64)    #$ error[E3100]
    unsafe:  # SAFETY(ffi): the cases the checker refuses
        untyped = v.sum_wide(1, 5)    #$ error[E5015]
        promoted = v.sum_wide(1, narrow)    #$ error[E5013]: C passes `narrow` as a 4-byte `int` through `...`; write the size `sum_wide` expects: `narrow as c_int` for 4 bytes, or `narrow as i64` for 8
        doubled = printf(c"%f", single)    #$ error[E5013]: C passes `single` as an 8-byte `double` through `...`; write `single as f64`
        flagged = v.sum_wide(1, flag)    #$ error[E5013]: C passes `flag` as a 4-byte `int` through `...`; write the size `sum_wide` expects: `(1 if flag else 0) as c_int` for 4 bytes, or `(1 if flag else 0) as i64` for 8
        lettered = v.sum_wide(1, letter)    #$ error[E5015]: a `char` does not cross to C; pass its scalar value: `letter as u32`
        missing = v.sum_wide()    #$ error[E2020]
    held = printf    #$ error[E5015]
