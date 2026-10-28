#$ test: run-pass
#$ rules: R4
#$ stdout: 1
## R4: `ref mut c: Counter` names the caller's place, and every change through `c` reaches the
## caller's value; the call says the same words, `bump(ref mut counter)` (AN-20, verdict 1).

class Counter:
    n: int = 0

fn bump(ref mut c: Counter):
    c.n += 1

fn main():
    counter = Counter()
    bump(ref mut counter)
    println(counter.n)
