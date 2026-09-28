#$ test: run-fail
#$ rules: HEAP-8, ALC-4
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ panics: capacity overflow
# `[HEAP-8]` — 2^62 more `int`s is 2^65 bytes, past what `usize` holds: the
# request panics with `capacity overflow` before anything is allocated,
# instead of wrapping to a small buffer the pushes would then overrun.

fn main():
    xs: Array[int] = []
    println(xs.len())
    xs.reserve(4611686018427387904)
    xs.push(1)
    println(xs.len())
