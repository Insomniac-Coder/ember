#$ test: run-pass
#$ rules: STR-3, DRV-1, OWN-8
#$ stdout: 1 1 1
## `[STR-3]` - `@derive(Copy)` requests `Copy`: `=` from a whole name copies, and the struct has `Clone`
## too, so `copy of` works on it.

@derive(Copy)
struct P:
    x: int
    y: int

fn main():
    p = P(1, 2)
    q = p
    c = copy of p
    println(p.x, q.x, c.x)
