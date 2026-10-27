#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 8
#$ stdout: 2
## Verdicts 38, 50 and 86, R12 (rules.md lines 703-711): a function that calls a function right away may
## take it as a template parameter (`[F: fn(int)]` above it: `F` names the exact function given, one
## instance each, called directly) or as a function value (`owns f: fn(int)`, called through its two
## words); "a program's own functions may write either". Each call makes what its function changes,
## here `main`'s own `total` and `seen`, worked out with no word.

[F: fn(int)]
fn apply_each(xs: Array[int], owns f: F):
    for x in xs:
        f(x)

fn apply_each_value(xs: Array[int], owns f: fn(int)):
    for x in xs:
        f(x)

fn main():
    xs = [3, 5]
    total = 0
    apply_each(xs, fn(x): total += x)
    seen: Array[int] = []
    apply_each_value(xs, fn(x): seen.push(x))
    println(total)
    println(seen.len())
