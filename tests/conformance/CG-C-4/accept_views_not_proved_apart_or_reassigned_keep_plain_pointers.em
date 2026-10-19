#$ test: run-pass
#$ rules: CG-C-4, SIMD-3
#$ profiles: release
#$ stdout: 1.0 3.0 5.0
#$ 3.0
#$ assert-c: !contains("* restrict ")
# `[CG-C-4]` — a view gets a `restrict` pointer only when `[SIMD-3]` proves it apart from the
# loop's others, "and a view reassigned inside the loop is not hoisted". `add`'s two views are
# parameters, which may view one list as far as the function knows ("neither are parameter
# modes" a proof); `first_of_each` sets `xs` in its loop. Nothing in the emitted C is `restrict`.

@noinline
fn add(xs: Span[f64], mut out: MutSpan[f64]):
    for i in 0..xs.len():
        out[i] = out[i] + xs[i]

@noinline
fn first_of_each(a: Array[f64], b: Array[f64]) -> f64:
    xs: Span[f64] = a
    total = 0.0
    for i in 0..2:
        total += xs[i]
        xs = b
    return total

fn main():
    a: Array[f64] = []
    b: Array[f64] = []
    for i in 0..3:
        a.push(i as f64)
        b.push(1.0 + i as f64)
    add(a, b)
    println(b[0], b[1], b[2])
    println(first_of_each(a, b))
