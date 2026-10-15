#$ test: compile-fail
#$ rules: CT-3
#$ error[E6001]: evaluating `FOREVER` takes more than 100000000 steps
#$ help: raise the limit with `steps = …` under `[comptime]` in the manifest
#$ error[E6001]: evaluating `HUGE` takes more than 268435456 bytes of heap
#$ help: raise the limit with `heap = …` under `[comptime]` in the manifest
# `[CT-3]` — passing the step limit or the heap limit is `E6001`; `[comptime]` in the manifest
# raises either.

fn spin() -> int:
    n = 0
    while true:
        n += 1
    return n

fn hoard() -> int:
    xs: Array[int] = []
    xs.reserve(100_000_000)
    return xs.len()

const FOREVER: int = spin()
const HUGE: int = hoard()

fn main():
    println(FOREVER, HUGE)
