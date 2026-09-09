#$ test: parse-fail
#$ rules: GRM-23
# The other side of the same check: an ordinary chained comparison is still
# `E0102`, so giving `[GRM-23]` its `E0104` did not move every non-associative
# rejection onto one code.

fn main():
    b = 1 < 2 < 3               #$ error[E0102]: chained comparison
    println(1)
