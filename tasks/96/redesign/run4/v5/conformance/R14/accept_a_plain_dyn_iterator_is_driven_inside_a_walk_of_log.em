#$ test: run-pass
#$ rules: R12, R14, R8
#$ stdout: 33
## R12 and R8 (c) (verdict 86): driving `it` makes the changes of every iterator the program puts there;
## here only `xs.iter()`, which changes nothing, so driving it inside a walk of `LOG` is allowed. The
## same walk given an adapter that pushes to `LOG` is refused
## (`reject_a_dyn_that_pushes_to_log_driven_inside_a_walk_of_log.em`).

static mut LOG: Array[int] = [1, 2]

fn add_each(owns it: owns dyn Iterator[Item = int]) -> int:
    total = 0
    for entry in LOG:
        if it.next() exists as v:
            total += v + entry
    return total

fn main():
    xs = [10, 20]
    println(add_each(xs.iter()))
