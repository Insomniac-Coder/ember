#$ test: run-pass
#$ rules: TYP-14
#$ stdout: 6 25 true
# `[TYP-14]` — a `Box[T]` is read through wherever a `T` is wanted, an
# operand included.

b: owns i64 = owns 5
println(b + 1, b * b, b > 4)
