#$ test: run-pass
#$ rules: OWN-9
#$ stdout: a 2
#$ stdout: 1 2
# `[OWN-9]` — `[copy of a, b]` keeps `a` and hands `b` over; a `Copy` value is copied.

fn main():
    a = String("a")
    b = String("b")
    xs = [copy of a, b]
    println(a, xs.len())
    n = 1
    ns = [n, 2]
    println(n, ns[1])
