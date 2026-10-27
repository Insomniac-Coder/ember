#$ test: compile-fail
#$ rules: R12, R8, R14
#$ help: move line 21 below the loop
## Verdicts 46 and 86, R12 (rules.md lines 711-715): "driving an `owns dyn Iterator[Item = int]` makes the
## changes of every adapter the program can put there"; the `dyn` type says nothing about changes, and
## the compiler works out that `show` drives an adapter whose steps push to `LOG`, so the call inside a
## loop over `LOG` is refused (E3020), the message naming the chain (R18). The nearest allowed program is
## accept_driving_a_dyn_iterator_that_pushes_to_log_after_the_loop.

static mut LOG: Array[int] = []

fn show(owns it: owns dyn Iterator[Item = int]):
    for x in it:
        println(x)

fn main():
    LOG.push(0)
    xs = [1, 2]
    for entry in LOG:
        println(entry)
        show(xs.iter().inspect(fn(x): LOG.push(x)))    #$ error[E3020]: `show(...)` (line 21) adds to `LOG` (`show` drives the iterator it is given, line 13, and each step runs the function `inspect` keeps, which pushes to `LOG`) while this loop walks `LOG` (line 19); call it after the loop: move line 21 below the loop
