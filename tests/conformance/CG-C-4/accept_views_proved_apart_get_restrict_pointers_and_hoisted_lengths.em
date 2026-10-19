#$ test: run-pass
#$ rules: CG-C-4, SIMD-3
#$ profiles: development
#$ stdout: 20.0
#$ 14.0
#$ assert-c: contains("double* restrict _")
#$ assert-c: contains("_rn = ")
# `[CG-C-4]` — "the base pointer of every view whose base and length are invariant in the loop is
# hoisted into a local declared `T *restrict` exactly when `[SIMD-3]` … proves it disjoint from
# every other view the loop writes or reads; its length is hoisted into a local as well". Two lists
# the function owns are apart by `[SIMD-3]` (b), and two columns of one `SoA` by (c): each loop
# below reads its two through `restrict` pointers set before it, and compares its indices with
# lengths read once there. (Neither loop is moved into a function of its own: they write nothing.)

struct Sample:
    x: f64
    w: f64

@noinline
fn dot(n: int) -> f64:
    a: Array[f64] = []
    b: Array[f64] = []
    for i in 0..n:
        a.push(i as f64)
        b.push(2.0)
    total = 0.0
    for i in 0..n:
        total += a[i] * b[i]
    return total

@noinline
fn weighted(n: int) -> f64:
    ps = SoA[Sample].with_capacity(n)
    for i in 0..n:
        ps.push(Sample(x=i as f64, w=i as f64))
    xs = ps.x
    ws = ps.w
    total = 0.0
    for i in 0..n:
        total += xs[i] * ws[i]
    return total

fn main():
    println(dot(5))
    println(weighted(4))
