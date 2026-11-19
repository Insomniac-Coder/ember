#$ test: run-pass
#$ rules: DET-6, DET-1
#$ stdout: 6
# `[DET-6]` — `@deterministic` constrains results, not timing: taking a lock changes when the
# function runs, never what it returns, so a `@deterministic` function may take one.

from std.sync import Mutex

static TOTAL: Mutex[int] = Mutex(0)

@deterministic
fn add(xs: Span[int]) -> int:
    with total = TOTAL.lock():
        for x in xs:
            total += x
        return total

fn main():
    xs = [1, 2, 3]
    println(add(xs.as_span()))
