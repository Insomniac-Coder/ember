#$ test: run-pass
#$ rules: R12, R14, R5
#$ stdout: 1
#$ stdout: 2
#$ stdout: 2
## R12 and R5 (verdict 86): a `dyn` type says nothing about what its values' methods change; driving
## `it` makes the changes of every adapter the program can put there, worked out from the program (here
## the function given to `inspect`, which pushes to `LOG`), and no first line says it. Renamed from
## step 3's `accept_a_dyn_iterator_type_says_what_its_steps_change.em`.

static mut LOG: Array[int] = []

fn show(owns it: owns dyn Iterator[Item = int]):
    for x in it:
        println(x)

fn main():
    xs = [1, 2]
    show(xs.iter().inspect(fn(x): LOG.push(x)))
    println(LOG.len())
