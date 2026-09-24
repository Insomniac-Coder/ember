#$ test: run-fail
#$ rules: TYP-30
#$ panics: `**` with a negative exponent
# `[TYP-30]` — a negative exponent that is not a constant panics.

e = -1
println(2 ** e)
