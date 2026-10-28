#$ test: run-pass
#$ rules: R12, R14
#$ stdout: 1
#$ stdout: 2
#$ stdout: 2
## R12 (verdict 46): "A `dyn` type says what its values' methods may change, as a function type does:
## `owns dyn Iterator[Item = int] changes LOG`, and a function that drives it names the change".

static mut LOG: Array[int] = []

fn show(owns it: owns dyn Iterator[Item = int] changes LOG) changes LOG:
    for x in it:
        println(x)

fn main() changes LOG:
    xs = [1, 2]
    show(xs.iter().inspect(fn(x) changes LOG: LOG.push(x)))
    println(LOG.len())
