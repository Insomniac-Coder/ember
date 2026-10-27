#$ test: run-pass
#$ rules: R4, R12
#$ stdout: 1
## R4: a callable type writes the same words as a parameter (`fn(ref mut Counter)`), and a call through
## a function value says `ref mut` before the argument it may change.

class Counter:
    n: int = 0

fn bump(ref mut c: Counter):
    c.n += 1

fn apply(f: fn(ref mut Counter), ref mut c: Counter):
    f(ref mut c)

fn main():
    counter = Counter()
    apply(bump, ref mut counter)
    println(counter.n)
