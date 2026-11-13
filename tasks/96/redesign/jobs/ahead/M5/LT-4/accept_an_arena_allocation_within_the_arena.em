# Clause LT-4.5: "Arena allocations point into the arena (`[ARN-1]`)."
#$ test: run-pass
#$ rules: LT-4, ARN-1, LT-44
#$ stdout: 7 5
# `build`'s result points into the arena it is given (`[LT-44]`), which outlives every use.

fn build(ref mut frame: Arena) -> ref mut int:
    return frame.alloc(owns 7)

fn main():
    frame = Arena.with_capacity(64)
    a = build(ref mut frame)
    b = frame.alloc(owns 5)
    println(a, b)
