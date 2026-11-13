#$ test: run-pass
#$ rules: GRM-47, GRM-43
#$ stdout: 1
#$ stdout: 5
## `[GRM-47]` — a computed value handed over goes in brackets; `owns` on a value alone is no operand;
## the other words make a value and end nothing: `copy of a + b` adds a copy of `a` to `b`.

fn main():
    m: Map[int, int] = Map()
    i = 3
    m.insert(owns (i * 7), owns i)
    println(m.len())
    a = 2
    b = 3
    s = copy of a + b
    println(s)
