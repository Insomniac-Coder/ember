#$ test: run-pass
#$ rules: STD-26
#$ stdout: 10 24 true true 3 1.75
# `[STD-26]` — `sum(it, start=0)`, `any(it)` and `all(it)` consume an iterable; `sum` adds from
# `start`, left to right.

fn main():
    xs = [1, 2, 3]
    a = sum(0..5)
    b = sum((x * x for x in xs), 10)
    c = any(x > 2 for x in xs)
    g = (x > 0 for x in xs)
    d = all(g)
    e = sum(enumerate(xs).map(fn(p) => p.0))
    fs = [0.5, 0.25]
    f = sum(fs.iter(), 1.0)
    println(a, b, c, d, e, f)
