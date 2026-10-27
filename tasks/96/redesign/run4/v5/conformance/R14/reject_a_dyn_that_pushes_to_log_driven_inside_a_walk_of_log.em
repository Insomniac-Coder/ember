#$ test: compile-fail
#$ rules: R12, R14, R8
## R12 and R8 (c) (verdict 86): driving `it` runs the function `main` gives to `inspect`, which pushes to
## `LOG`, worked out from the program (no `dyn` type says it); inside a walk of `LOG` that is E3020, the
## message naming the chain. Its way out walks a copy of its own, which nothing restricts (R8 (g)).
## Step 3's `reject_a_dyn_that_changes_log_driven_inside_a_walk_of_log.em`, renamed.

static mut LOG: Array[int] = [1, 2]

fn add_each(owns it: owns dyn Iterator[Item = int]) -> int:
    total = 0
    for entry in LOG:
        if it.next() exists as v:    #$ error[E3020]: `it.next()` (line 13) pushes to `LOG` (through the function `main` gives to `inspect` on line 19, which each step of `it` runs) while this loop walks `LOG` (line 12); walk a copy of your own: write `for entry in copy of LOG:` in place of line 12
            total += v + entry
    return total

fn main():
    xs = [10, 20]
    println(add_each(xs.iter().inspect(fn(x): LOG.push(x))))
