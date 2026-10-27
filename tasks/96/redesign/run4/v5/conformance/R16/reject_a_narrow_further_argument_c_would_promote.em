#$ test: compile-fail
#$ rules: FFI-8
## design rules: R16
## Section 3's E5013 row: "a further argument of a variadic C function that C would promote to another
## type" (AN-84; completeness G2). C passes an `i16` further argument as an `int`; the message is the
## lead's (worklist section 3), in this program's names and lines: it names only the promotion, since
## what `sum_wide` reads is in C, which the compiler cannot see.

import c "./wide_sum.c" as v

fn main():
    narrow: i16 = 3
    unsafe:  # SAFETY(ffi): sum_wide reads each further argument as an i64
        println(v.sum_wide(1, narrow))    #$ error[E5013]: C passes `narrow` as an `int` through `...`; if `sum_wide` reads another type, convert it first: write `println(v.sum_wide(1, narrow as i64))` in place of line 13
