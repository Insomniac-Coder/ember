#$ test: run-pass
#$ rules: SIMD-5, SIMD-2, PAR-2b
#$ profiles: debug, release
#$ stdout: [4, 8, 12, 16]
#$ 1.5
#$ [3, 7]
#$ 6.5
# `[SIMD-5]` — "every call in the body is to a function `[CG-C-3]` places in
# the inline header, read through as `[PAR-2b]` says": a call to a small
# function, an operator of a small struct and a method do not keep a loop out
# of vectorisable form, so `@simd(assert)` accepts each of these loops. And
# "every floating-point reduction is declared by `@parallel(reduce=…)`": a
# float total the parallel loop declares is in vectorisable form too, where
# the same total undeclared is not (`reject_loops_not_in_vectorisable_form`).
# Reading through changes nothing a program sees.
from std.math import Vec3

struct Pair:
    x: int
    y: int

extend Pair:
    fn sum(self) -> int:
        return self.x + self.y

fn helper(x: int) -> int:
    return x * 2 + 1

# Called from two places in one loop, so it is not simply folded into the
# caller as a function called once is.
fn calls(xs: Span[int], mut out: MutSpan[int]):
    @simd(assert)
    for i in 0..xs.len():
        out[i] = helper(xs[i]) + helper(xs[i] - 1)

fn moved(mut ps: MutSpan[Vec3], vs: Span[Vec3], dt: f32):
    @simd(assert)
    for i in 0..ps.len():
        ps[i] += vs[i] * dt

fn sums(ps: Span[Pair], mut out: MutSpan[int]):
    @simd(assert)
    for i in 0..ps.len():
        out[i] = ps[i].sum()

fn total(xs: Span[float]) -> float:
    s = 0.0
    @simd(assert)
    @parallel(reduce=[s: +])
    for i in 0..xs.len():
        s += xs[i]
    return s

fn main():
    a = [1, 2, 3, 4]
    b = [0, 0, 0, 0]
    calls(a, b)
    println(b)
    ps = [Vec3(1.0, 2.0, 3.0)]
    vs = [Vec3(1.0, 1.0, 1.0)]
    moved(ps, vs, 0.5)
    println(ps[0].x)
    qs = [Pair(x=1, y=2), Pair(x=3, y=4)]
    o = [0, 0]
    sums(qs, o)
    println(o)
    println(total([1.0, 2.0, 3.5]))
