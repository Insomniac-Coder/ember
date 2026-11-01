#$ test: run-pass
#$ rules: OWN-9, GRM-15, OWN-3
#$ stdout: 5 7
#$ 6
#$ 1 2
#$ 3 2
#$ 0
#$ 1
# `[OWN-9]` — `b = owns n` hands a number over: a name declared with a type too, and a struct's
# part, which is given a new value before its owner is used whole (`[OWN-3]`). A name handed over
# is given a new value as any other; a counted loop's turn hands its own number over.

struct Point:
    x: int
    y: int

fn main():
    n = 5
    b = owns n
    k: i64 = 7
    c: i64 = owns k
    println(b, c)
    n = 6
    println(n)
    p = Point(x=1, y=2)
    x = owns p.x
    println(x, p.y)
    p.x = 3
    println(p.x, p.y)
    for i in 0..2:
        kept = owns i
        println(kept)
