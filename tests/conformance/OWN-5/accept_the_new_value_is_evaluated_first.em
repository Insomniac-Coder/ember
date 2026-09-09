#$ test: run-pass
#$ rules: OWN-5, OWN-2
# "Overwriting a place that holds a live value drops the old value first
# (after evaluating the new value: `x = f(x)` moves `x` into `f`, then
# stores)." The order matters: evaluating the new value may read the old one,
# so the drop cannot come first.

fn grow(xs: Array[i32]) -> Array[i32]:
    ys: Array[i32] = Array[i32]()
    ys.push(xs[0] + 10)
    return ys

fn main():
    x: Array[i32] = Array[i32]()
    x.push(1)
    x = grow(x)
    println(x[0])
#$ stdout: 11
