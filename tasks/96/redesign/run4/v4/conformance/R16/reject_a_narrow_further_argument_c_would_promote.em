#$ test: compile-fail
#$ rules: R16
## Section 4: E5013, "a variadic further argument C would promote" (AN-84; completeness G2). C passes an
## `i16` further argument as an `int`; the message is the worklist's (section 4, item 12), in this
## program's names.

import c "./wide_sum.c" as v

fn main():
    narrow: i16 = 3
    unsafe:  # SAFETY(ffi): sum_wide reads each further argument as an i64
        println(v.sum_wide(1, narrow))    #$ error[E5013]: C passes the extra argument `narrow` as an `int`; `sum_wide` reads an `int64_t`; pass it as one: `narrow as i64`
