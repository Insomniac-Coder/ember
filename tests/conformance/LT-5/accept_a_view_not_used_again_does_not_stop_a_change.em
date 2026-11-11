#$ test: run-pass
#$ rules: LT-5
#$ stdout: 1
#$ stdout: 2
# `[LT-5]` — regions of locals are inferred by non-lexical liveness: a view whose last use is
# behind us does not hold its owner.

fn main():
    xs: Array[i32] = Array[i32]()
    xs.push(owns 1)
    first = xs[0]
    v = ref first
    println(v)
    xs.push(owns 2)
    println(xs.len())
