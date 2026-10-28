#$ test: run-pass
#$ rules: R16
#$ stdout: 3
## Section 4: the line E5013 gives: the further argument is passed as the `int64_t` C reads (completeness
## G2's nearest allowed).

import c "./wide_sum.c" as v

fn main():
    narrow: i16 = 3
    unsafe:  # SAFETY(ffi): sum_wide reads each further argument as an i64
        println(v.sum_wide(1, narrow as i64))
