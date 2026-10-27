#$ test: run-pass
#$ rules: R14, R2
#$ stdout: 3
## R14: the nearest allowed program to a kind test that always passes: a value already a `Circle` is
## named with `=` (a read-only view, R2), as E2036's message says.

struct Circle:
    r: int

fn total(circles: Array[Circle]) -> int:
    t = 0
    for s in circles:
        c = s
        t += c.r
    return t

fn main():
    println(total([Circle(r=1), Circle(r=2)]))
