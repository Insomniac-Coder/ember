#$ test: run-pass
#$ rules: PHIL-8, BRW-5
#$ stdout: [10, 20]
# `[PHIL-8]` — where a property can be proved statically, it is: two mutable views of one array,
# proved apart by `split_at`, are written with no run-time check of their overlap.

fn set(mut s: MutSpan[int], v: int):
    s[0] = v

fn main():
    xs = [0, 0]
    left, right = xs.as_mut_span().split_at(1)
    set(left, 10)
    set(right, 20)
    println(xs)
