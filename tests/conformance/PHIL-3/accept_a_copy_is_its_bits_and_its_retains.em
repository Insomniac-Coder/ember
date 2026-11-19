#$ test: run-pass
#$ rules: PHIL-3, OWN-7
#$ profiles: debug, development
#$ stdout: 3 2 2
#$ assert-c: !contains(em_4Pair5clone)
# `[PHIL-3]` — a non-`Copy` value is copied only when the program says so (`clone`), and a copy
# of a `Copy` value is its bits, with each class handle in it retained: no other code runs, so the
# object is dropped once, when the last copy goes.

@derive(Copy)
struct Pair:
    v: int
    n: int

fn main():
    a = [1, 2, 3]
    b = copy of a
    p = Pair(2, 1)
    q = p
    println(a.len() + b.len() - 3, p.v, q.v)
