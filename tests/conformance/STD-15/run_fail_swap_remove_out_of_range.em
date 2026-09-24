#$ test: run-fail
#$ rules: STD-15, ERR-13
#$ panics: swap_remove index out of range
# `[STD-15]` — `swap_remove(i)` needs an element at `i`; a caller's bug
# panics (`[ERR-13]`).

xs = [1, 2]
println(xs.swap_remove(2))
