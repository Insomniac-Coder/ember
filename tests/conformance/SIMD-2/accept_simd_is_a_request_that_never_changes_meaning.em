#$ test: run-pass
#$ rules: SIMD-2
#$ profiles: debug, release, shipping
#$ stdout: [3, 6, 9, 12]
#$ 11
#$ 6.5
#$ 3
#$ 2
# `[SIMD-2]` — "`@simd` on a `for` loop is a request and a report ... It never
# changes meaning." A loop in vectorisable form and loops that are not (an
# indirect index, a float total no reduction declares, a text loop, an early
# exit) all run as written; only `@simd(assert)` makes failure an error.

fn scale(xs: Span[int], mut out: MutSpan[int], k: int):
    @simd
    for i in 0..xs.len():
        out[i] = xs[i] * k

fn gathered(xs: Span[int], idx: Span[int]) -> int:
    s = 0
    @simd
    for i in 0..idx.len():
        s += xs[idx[i]]
    return s

fn total(xs: Span[float]) -> float:
    s = 0.0
    @simd
    for x in xs:
        s += x
    return s

fn count(t: str, wanted: char) -> int:
    n = 0
    @simd
    for c in t.chars():
        if c == wanted:
            n += 1
    return n

fn first_at_least(xs: Span[int], limit: int) -> int:
    @simd
    for i in 0..xs.len():
        if xs[i] >= limit:
            return i
    return -1

fn main():
    a = [1, 2, 3, 4]
    b = [0, 0, 0, 0]
    scale(a, b, 3)
    println(b)
    println(gathered(a, [3, 3, 1, 0]))
    println(total([1.0, 2.0, 3.5]))
    println(count("banana", 'a'))
    println(first_at_least(a, 3))
