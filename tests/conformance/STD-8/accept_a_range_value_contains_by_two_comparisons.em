#$ test: run-pass
#$ rules: STD-8, STD-26, CTL-3
#$ stdout:
#$ 4 3 true false true true
#$ 5 true 5 false true
#$ 0 1 0 256
#$ true false true false
#$ 6 true
# `[STD-8]` — a `Range[T]` implements `Contains` "as two comparisons", a
# `RangeFrom` and a `RangeTo` as one; `[STD-26]` — `len(r)` of `a..b` or
# `a..=b` is how many values it has, and `r.len()` is the same. Both work on a
# range held in a variable, not only one written in place (ODR-027).

r = 2..6
println(len(r), len(0..3), 3 in r, 9 in r, 4 in 2..=4, 7 not in r)
q = 2..=6
println(len(q), 6 in q, q.len(), q.contains(7), r.contains(2))
println(len(5..2), len(5..=5), len(5..=4), len(0u8..=255))
println(3 in ..4, 4 in ..4, 10 in 7.., 6 in 7..)
unit = 0.5..1.5
println(len(-3..3), 0.7 in unit)
