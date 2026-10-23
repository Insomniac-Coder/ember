#$ test: run-pass
#$ rules: FN-6, MOD-2, GRM-24
#$ profiles: debug, development, release
#$ stdout: 2 4 12 11
# `[FN-6]` — a function is a value wherever its name reaches it, through its module too
# (`steps.offset`, as `steps.twice(…)` calls one): it was `E1010`, "`steps` is a module, and what
# follows it is not one of its items". Its type is its own (ODR-278), so a generic given it is
# made for it, and it converts to `fn(int) -> int` where that is written.

import items.steps

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn main():
    f = steps.offset
    println(f(1), steps.twice(steps.offset, 2), apply(steps.offset, 11), apply(f, 10))
