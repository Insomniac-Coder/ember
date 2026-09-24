#$ test: run-fail
#$ rules: TYP-30, TYP-8
#$ panics: integer overflow
# `[TYP-30]` — an integer power overflows per `[TYP-8]`: `3 ** 6` is 729,
# which a `u8` cannot hold.

x: u8 = 3
println(x ** 6)
