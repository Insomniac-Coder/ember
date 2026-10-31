#$ test: run-pass
#$ rules: GRM-44, GRM-13
#$ stdout: 7
#$ stdout: none
#$ stdout: 12
#$ stdout: 11
#$ stdout: 2 5
## `[GRM-44]` — `if e exists as p:` runs its block when the `Option` `e` holds a value, which it
## names `p` there, and `elif` and `else` follow as for any `if`; `while e exists as x:` repeats
## while `e` holds one, naming it each round. `exists as ref mut c` may change what it names, and
## an `Option` of a pair is opened with a word on each name.

struct Counter:
    n: int = 0

fn total(xs: Array[int]) -> int:
    seen = 0
    it = xs.iter()
    while it.next() exists as x:
        seen += x
    return seen

fn main():
    a: Option[int] = Some(7)
    if a exists as v:
        println(v)
    b: Option[int] = None
    if b exists as v:
        println(v)
    elif b exists as w:
        println(w)
    else:
        println("none")
    println(total([3, 4, 5]))
    held: Option[Counter] = Some(Counter(n=10))
    if held exists as ref mut c:
        c.n += 1
    if held exists as c:
        println(c.n)
    pair: Option[(int, int)] = Some((2, 5))
    if pair exists as (x, ref y):
        println(x, y)
