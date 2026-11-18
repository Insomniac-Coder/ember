#$ test: run-pass
#$ rules: FN-1a, FN-2a
#$ stdout: [1.5, 2.5]
#$ stdout: [2.0, 3.0]
#$ stdout: 9
# `[FN-1a]` — a `MutSpan[T]` parameter that writes the elements it views says `ref mut`; at the call
# the word stands before the span's elements, so it may stand before a span made in the call, and a
# `const ref mut` one writes the elements too.

struct Particles:
    xs: Array[float]

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in range(src.len()):
        dst[i] += src[i]

fn bump_all(const ref mut dst: MutSpan[int]):
    for i in range(dst.len()):
        dst[i] += 1

fn main():
    p = Particles(xs=[1.0, 2.0])
    v = [0.5, 0.5]
    blend(ref mut p.xs.as_mut_span(), v.as_span())
    println(p.xs)
    d = ref mut p.xs.as_mut_span()
    blend(ref mut d, v.as_span())
    println(p.xs)
    ns = [1, 2, 3]
    bump_all(const ref mut ns.as_mut_span())
    println(ns[0] + ns[1] + ns[2])
