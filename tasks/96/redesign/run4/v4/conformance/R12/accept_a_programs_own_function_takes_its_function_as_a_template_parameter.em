#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 8
#$ stdout: 2
## Verdicts 38 and 50, R12 (rules.md lines 659-668): a function that calls a function right away may take
## it as a template parameter (`F` names the exact function given: one instance each, called directly)
## or as a function value (`fn(int) changes C`, called through its two words); both are legal, and both
## fill `C` from the function given, here `main`'s own `total` and `seen` (no clause in `main`).

[C, F: fn(int) changes C]
fn apply_each(xs: Array[int], owns f: F) changes C:
    for x in xs:
        f(x)

[C]
fn apply_each_value(xs: Array[int], owns f: fn(int) changes C) changes C:
    for x in xs:
        f(x)

fn main():
    xs = [3, 5]
    total = 0
    apply_each(xs, fn(x) changes total: total += x)
    seen: Array[int] = []
    apply_each_value(xs, fn(x) changes seen: seen.push(x))
    println(total)
    println(seen.len())
