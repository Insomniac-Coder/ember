#$ test: run-pass
#$ rules: R12, R14
#$ stdout: 1
#$ stdout: 2
#$ stdout: 2
## Verdict 46, R12 (rules.md lines 618-623): a `dyn` type says what its values' methods may change, and a
## function that drives it names the change; `inspect`'s adapter runs the lambda at each step.

static mut LOG: Array[int] = []

fn show(owns it: owns dyn Iterator[Item = int] changes LOG) changes LOG:
    for x in it:
        println(x)

fn main() changes LOG:
    xs = [1, 2]
    it = xs.iter().inspect(fn(x) changes LOG: LOG.push(x))
    show(it)
    println(LOG.len())
