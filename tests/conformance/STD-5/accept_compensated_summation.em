#$ test: run-pass
#$ rules: STD-5
#$ profiles: debug, release, shipping
#$ stdout: 2.0 0.0
#$ 1.0 0.9999999999999999 true
#$ 0.0 0.0 1e+100
# `[STD-5]`, ODR-043 — `math.KahanSum` keeps what each addition drops and
# adds it back at the end (Neumaier's form, which also holds when an addend
# is larger than the sum so far). `1 + 1e100 + 1 - 1e100` is 2; added in
# order it is 0, both ones lost to the big number. Ten `0.1`s make 1.0.

from std.math import KahanSum

fn main():
    k = KahanSum.new()
    plain = 0.0
    for x in [1.0, 1e100, 1.0, -1e100]:
        k.add(x)
        plain += x
    println(k.value(), plain)
    tenth: Array[f64] = [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]
    by_hand = 0.0
    for x in tenth:
        by_hand += x
    println(KahanSum.of(tenth.as_span()), by_hand, sum(tenth) == by_hand)
    empty: Array[f64] = []
    big = KahanSum.new()
    big.add(1e100)
    println(KahanSum.new().value(), KahanSum.of(empty.as_span()), big.value())
