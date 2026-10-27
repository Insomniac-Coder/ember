#$ test: run-pass
#$ rules: R4, R2
#$ stdout: 2.0 4.0
## R4, verdict 42: `const ref mut dst: MutSpan[float]` writes the caller's elements and is never given
## another ref, so the caller's span is the same span after the call; the call writes the same words,
## `blend(const ref mut d, ...)` (verdict 70).

fn blend(const ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn main():
    a = [1.0, 2.0]
    b = [3.0, 6.0]
    d = ref mut a.as_mut_span()
    blend(const ref mut d, b.as_span())
    println(a[0], a[1])
