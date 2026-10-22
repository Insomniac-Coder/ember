#$ test: compile-fail
#$ rules: FFI-8
# `[FFI-8]` — a variadic C function is called only in `unsafe` (`E5002`); each further argument has
# the type the program gives it, one C passes unchanged (`E5015`: an untyped literal, a type C's
# default argument promotions change); the function is called, never a value.

from std.ffi import c_int
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v

unsafe extern "C":
    @ffi(variadic, param(format, borrowed, nul_terminated))
    @safety("the format reads what each call passes")
    fn printf(format: cstr) -> c_int

fn main():
    narrow: i16 = 3
    single: f32 = 1.5
    total = v.sum_wide(1, 5i64)    #$ error[E5002]
    unsafe:  # SAFETY(ffi): the cases the checker refuses
        untyped = v.sum_wide(1, 5)    #$ error[E5015]
        promoted = v.sum_wide(1, narrow)    #$ error[E5015]
        doubled = printf(c"%f", single)    #$ error[E5015]
        missing = v.sum_wide()    #$ error[E2020]
    held = printf    #$ error[E5015]
