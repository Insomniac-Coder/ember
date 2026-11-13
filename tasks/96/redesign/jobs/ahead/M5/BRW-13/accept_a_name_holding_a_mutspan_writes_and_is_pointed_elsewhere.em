# Clause BRW-13.5: "A name that holds a `MutSpan` likewise: `d[i] = v` writes an element through it,
# and `d = ref mut other.as_mut_span()` points it at another span."
#$ test: run-pass
#$ rules: BRW-13, SPN-3
#$ stdout: 9 2 3 8

fn main():
    xs = [1, 2]
    ys = [3, 4]
    d = ref mut xs.as_mut_span()
    d[0] = 9
    d = ref mut ys.as_mut_span()
    d[1] = 8
    println(xs[0], xs[1], ys[0], ys[1])
