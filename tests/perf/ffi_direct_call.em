#$ perf: scalar
## `[BEN-6]` — a direct FFI call executes the instructions C's own call does: the program calls a C
## function 200 million times, as its C twin calls the same function. Neither side can inline it
## (the C is compiled apart, `noinline` in the twin), so each turn is one call on both sides; the
## call in the generated C is the C function's own (`milestones.rs`,
## `a_direct_ffi_call_is_the_c_functions_own_call`).

import c "ffi_direct_call_mix.c"

@overflow(wrap)
fn main():
    total: i64 = 0
    for i in 0..200000000:
        unsafe:  # SAFETY(ffi): a pure function of its arguments
            total = total + c.ffi_mix(i, 3)
    println(total)
