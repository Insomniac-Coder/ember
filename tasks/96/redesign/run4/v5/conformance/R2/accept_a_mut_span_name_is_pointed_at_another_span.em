#$ test: run-pass
#$ rules: R2
#$ stdout: 10 30
## R2: a name that holds a `MutSpan` likewise: `d[i] = v` writes through it, and
## `d = ref mut ys.as_mut_span()` points it at another span; nothing is written into `xs` by that
## line (verdicts 34, 42; AN-28).

fn main():
    xs = [1, 2]
    ys = [3, 4]
    d = ref mut xs.as_mut_span()
    d[0] = 10
    d = ref mut ys.as_mut_span()
    d[0] = 30
    println(xs[0], ys[0])
