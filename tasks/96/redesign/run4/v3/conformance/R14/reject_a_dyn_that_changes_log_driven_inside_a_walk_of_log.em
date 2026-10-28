#$ test: compile-fail
#$ rules: R12, R14, R8
## R12 (verdict 46): driving a `dyn` value whose type says `changes LOG` makes that change, so inside
## a walk of `LOG` it is refused (R8 (c), E3020), as a call through a function value is.

static mut LOG: Array[int] = [1, 2]

fn add_each(owns it: owns dyn Iterator[Item = int] changes LOG) changes LOG -> int:
    total = 0
    for entry in LOG:
        if it.next() exists as v:    #$ error[E3020]: this loop walks `LOG` (line 10), and `it.next()` (line 11) changes `LOG` (its type says `owns dyn Iterator[Item = int] changes LOG`); call `it.next()` before or after the loop
            total += v + entry
    return total

fn main() changes LOG:
    xs = [10, 20]
    println(add_each(xs.iter().inspect(fn(x) changes LOG: LOG.push(x))))
