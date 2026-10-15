#$ test: run-pass
#$ rules: STD-26, STD-19
#$ stdout: 1 10
#$ 2 20
#$ [11, 22]
#$ [2, 1, 0]
#$ 1 10
#$ 2 6
#$ 3 2
#$ 0 25
#$ 1 36
# `[STD-26]` — `enumerate(it, start)`, `zip(a, b)` and `reversed(x)` return lazy iterators, as
# Python 3's do, over anything a `for` loop iterates; in a `for` header too (`[STD-19]`).

fn main():
    xs = [10, 20]
    e = enumerate(xs, start=1)
    for i, x in e:
        println(i, x)
    z = zip(xs, [1, 2, 3])
    println([a + b for a, b in z])
    println([v for v in reversed(0..3)])
    for i, x in enumerate(range(10, 0, -4), start=1):
        println(i, x)
    for a, b in zip(0..2, (y * y for y in [5, 6, 7])):
        println(a, b)
