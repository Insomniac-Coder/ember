# Clause BRW-14.8: "A `ref mut` one writes the caller's place, and may be given another ref, as any
# `ref mut` name may: a `ref mut dst: MutSpan[float]` parameter given another span points the
# caller's `d` at it, and `ref mut c: Counter` given another ref points `c`, for the rest of the
# call, at that place."
#$ test: run-pass
#$ rules: BRW-14, BRW-13, LT-47, FN-1
#$ stdout: 1 2 3
#$ stdout: 1 10
# After `retarget`, the caller's `d` views `ys` (`[LT-47]`: it may point into another argument);
# after `bump_both`, `a` got the first change and `b` the second.

struct Counter:
    n: int

fn retarget(ref mut dst: MutSpan[int], ref mut other: Array[int]):
    dst[0] = 1
    dst = ref mut other.as_mut_span()
    dst[0] = 2

fn bump_both(ref mut c: Counter, ref mut spare: Counter):
    c.n += 1
    c = ref mut spare
    c.n += 10

fn main():
    xs = [0, 0]
    ys = [0, 0]
    d = ref mut xs.as_mut_span()
    retarget(ref mut d, ref mut ys)
    d[1] = 3
    println(xs[0], ys[0], ys[1])
    a = Counter(n=0)
    b = Counter(n=0)
    bump_both(ref mut a, ref mut b)
    println(a.n, b.n)
