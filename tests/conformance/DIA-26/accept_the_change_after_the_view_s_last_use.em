#$ test: run-pass
#$ rules: DIA-26, BRW-1
#$ stdout: 1
#$ stdout: [9, 2, 3]
# `[DIA-26]` — the one message's fix compiles: the change after the view's last use.

fn main():
    xs = [1, 2, 3]
    m = xs.as_mut_span()
    println(m[0])
    xs[0] = 9
    println(xs)
