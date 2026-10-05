#$ test: run-pass
#$ rules: STD-26, STD-19
#$ stdout: 18446744073709551615
# `[STD-26]`, G8-4 (the owner's design, 0.9.10) — `len` of a range is its count, in the type that
# holds every count a range of its kind can have: a `u64` for a `..` range of 64-bit numbers. It
# panicked before, "len: the range has more values than an `int` can hold".

big = 0u64..18446744073709551615
println(len(big))
