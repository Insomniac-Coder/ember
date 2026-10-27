#$ test: run-pass
#$ rules: R16
#$ stdout: 3
## Section 3's E5013 row: the line its message gives: the further argument is converted before the call,
## to the `int64_t` this C function reads (completeness G2's nearest allowed).

import c "./wide_sum.c" as v

fn main():
    narrow: i16 = 3
    unsafe:  # SAFETY(ffi): sum_wide reads each further argument as an i64
        println(v.sum_wide(1, narrow as i64))
