#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 2997 2997 0
#$ assert-c-count: contains(" >> 56ULL;") == 2
# Ordinary Array and Span iteration lends each element by reference. Reading
# through that reference is the same unit-stride access as xs[i], so both
# running totals have the block proof of SIMD-7, including an empty span.

fn span_total(xs: Span[int]) -> int:
    total = 0
    for x in xs:
        alias: ref int = x
        total += alias
    return total

fn array_total(xs: Array[int]) -> int:
    total = 0
    for x in xs:
        total += x
    return total

fn main():
    xs: Array[int] = []
    for i in 0..1000:
        xs.push(i % 7)
    empty: Array[int] = []
    println(array_total(xs), span_total(xs.as_span()), span_total(empty.as_span()))
