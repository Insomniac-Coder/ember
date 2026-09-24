#$ test: run-fail
#$ rules: STD-26
#$ panics: len: the range has more values than an `int` can hold
# `[STD-26]` — `len` is an `int`. A range with more values than an `int`
# holds panics, as Python's `len` raises, rather than wrapping.

big = 0u64..18446744073709551615
println(len(big))
