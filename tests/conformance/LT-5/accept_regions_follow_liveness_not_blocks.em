# Clause LT-5.1: "Regions of locals are inferred by non-lexical liveness (§XVIII.4)."
#$ test: run-pass
#$ rules: LT-5, BCK-2, BCK-6, BRW-2
#$ stdout: [9, 2, 3]
#$ stdout: [1, 3, 5]
# On the path where `v` is not used again, its list may change, though `v` is still in scope; and
# `[BCK-6]`'s `first_even` returns a view on one path while the other changes the span it views.

fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    v = xs.as_span()
    if xs.len() > 5:
        println(v[0])
    else:
        xs[0] = 9
    println(xs)
    ys = [1, 3, 4]
    r = ref mut first_even(ref mut ys)
    r += 1
    println(ys)
