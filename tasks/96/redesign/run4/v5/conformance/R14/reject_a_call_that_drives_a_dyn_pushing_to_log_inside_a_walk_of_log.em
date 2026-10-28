#$ test: compile-fail
#$ rules: BRW-18, CTL-2
## design rules: R12, R14, R8
## R12 and R8 (c) (verdict 86): "driving an `owns dyn Iterator[Item = int]` makes the changes of every
## adapter the program can put there", and a call makes the changes of what it drives; `show` drives
## `it`, here an adapter whose function pushes to `LOG`, so `show(...)` inside a walk of `LOG` is E3020,
## the message naming the chain. It replaces step 3's test of a word missing from `show`'s first line
## (E2105), which verdict 86 removed, with the check that is left. The nearest allowed program calls
## `show` outside the walk (`accept_a_dyn_iterator_whose_steps_push_to_log_needs_no_word.em`).

static mut LOG: Array[int] = [7]

fn show(owns it: owns dyn Iterator[Item = int]):
    for x in it:
        println(x)

fn main():
    xs = [1, 2]
    for entry in LOG:
        println(entry)
        show(xs.iter().inspect(fn(x): LOG.push(x)))    #$ error[E3020]: `show(...)` (line 20) pushes to `LOG` (through `show`'s walk of `it`, line 13, which runs the function given to `inspect` on line 20) while this loop walks `LOG` (line 18); call it after the loop: move line 20 below the loop
    println(LOG.len())
