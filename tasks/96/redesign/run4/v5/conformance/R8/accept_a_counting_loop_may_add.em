#$ test: run-pass
#$ rules: BRW-16
#$ stdout: [1, 2, 3, 10, 20, 30]
## design rules: R8
## R8 (c): a counting loop `for i in 0..xs.len():` has its range fixed when it starts, so it may add to
## `xs`: the new elements are not visited.

fn main():
    xs = [1, 2, 3]
    for i in 0..xs.len():
        xs.push(xs[i] * 10)
    println(xs)
