#$ test: run-pass
#$ rules: R12, R14
#$ stdout: 0
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3
## Verdicts 46 and 86, R12 (rules.md lines 711-715): `inspect`'s adapter keeps the lambda and runs it at
## each step; made into an `owns dyn Iterator[Item = int]`, whose type says nothing about changes, it is
## driven by `show` after the loop over `LOG`, and `show` writes nothing. The nearest allowed program to
## reject_driving_a_dyn_iterator_that_pushes_to_log_inside_a_loop_over_log.

static mut LOG: Array[int] = []

fn show(owns it: owns dyn Iterator[Item = int]):
    for x in it:
        println(x)

fn main():
    LOG.push(0)
    xs = [1, 2]
    for entry in LOG:
        println(entry)
    show(xs.iter().inspect(fn(x): LOG.push(x)))
    println(LOG.len())
