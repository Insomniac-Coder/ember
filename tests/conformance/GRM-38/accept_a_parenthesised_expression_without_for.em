#$ test: run-pass
#$ rules: GRM-38, GRM-29
#$ stdout: 9
#$ stdout: 14
## `[GRM-38]` — `(e)` without `for` is a parenthesised expression, not a generator; with `for` it is
## one, and as the only argument of a call it needs no extra parentheses.

fn main():
    x = 3
    println((x) * (x))
    xs = [1, 2, 3]
    println(sum(v * v for v in xs))
