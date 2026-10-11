#$ test: run-pass
#$ rules: DSJ-3, SIMD-3, DSJ-2
#$ profiles: debug, release, shipping
#$ stdout: apart [2.0, 4.0, 6.0, 8.0]
#$ stdout: split [10, 20, 3, 4]
#$ stdout: unproved [3.0, 6.0, 9.0, 12.0]
# `[DSJ-3]` — views `assert_disjoint` proved apart are `restrict` for the C
# compiler: the `Ok` arm's loop runs in a function of its own, for every C
# compiler (D-557). `[SIMD-3]` — that is derived, never assumed: the `Err`
# arm's views and two view parameters prove nothing, so their loops stay
# where they are (D-568). `split_at`'s halves are apart too.

from std.mem import assert_disjoint

@noinline
fn scale(mut dst: MutSpan[f32], src: Span[f32], k: f32) -> str:
    match assert_disjoint(dst, src):
        Ok((d, s)):
            for i in 0..d.len():
                d[i] = s[i] * k
            return "apart"
        Err((d, s)):
            for i in 0..d.len():
                d[i] = s[i] * k
            return "overlap"

@noinline
fn halves(mut xs: MutSpan[int]):
    (lo, hi) = xs.split_at(2)
    for i in 0..lo.len():
        lo[i] = hi[i] * 0 + (i + 1) * 10

@noinline
fn unproved(mut dst: MutSpan[f32], src: Span[f32]):
    for i in 0..dst.len():
        dst[i] = src[i] * 3.0

fn main():
    a: Array[f32] = [0.0, 0.0, 0.0, 0.0]
    b: Array[f32] = [1.0, 2.0, 3.0, 4.0]
    println(scale(a, b, 2.0), a)
    n: Array[int] = [1, 2, 3, 4]
    halves(n)
    println("split", n)
    unproved(a, b)
    println("unproved", a)
