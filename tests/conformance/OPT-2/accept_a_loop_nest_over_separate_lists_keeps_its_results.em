#$ test: run-pass
#$ rules: OPT-2, PHIL-5
#$ profiles: debug, release, shipping
#$ stdout:
#$ 2197000
#$ 2002000 1999000
#$ 49
#$ 30000.0
# For MSVC, a loop nest over separate lists runs in a function of its own
# whose list parameters are `restrict`, with the counts the program fixes
# written in (ADR-079). The results are the same as before, on every
# compiler: counts fixed in the program and counts known only while it
# runs (those two nests move), a value the nest sets and the program reads
# after it (`last`), and decimal numbers each round adds to (a nest reading
# back what it writes); those two stay where they are.

fn rounds_of(a: Array[int], b: Array[int], rounds: int, n: int) -> int:
    out: Array[int] = []
    for i in 0..n:
        out.push(0)
    for round in 0..rounds:
        for i in 0..n:
            out[i] = a[i] + b[i] + round
    total = 0
    for x in out:
        total += x
    return total

fn main():
    n = 1000
    a: Array[int] = []
    b: Array[int] = []
    out: Array[int] = []
    for i in 0..n:
        a.push(i)
        b.push(3 * i)
        out.push(0)
    for round in 0..200:
        for i in 0..n:
            out[i] = a[i] + b[i] + round
    total = 0
    for x in out:
        total += x
    println(total)

    println(f"{rounds_of(a, b, 5, n)} {rounds_of(a, b, 2, n)}")

    last = 0
    for round in 0..50:
        for i in 0..n:
            out[i] = a[i] * 2 + round
        last = round
    println(last)

    xs: Array[float] = []
    vs: Array[float] = []
    for i in 0..100:
        xs.push(0.0)
        vs.push(1.5)
    for step in 0..200:
        for i in 0..100:
            xs[i] = xs[i] + vs[i]
    sum = 0.0
    for x in xs:
        sum += x
    println(sum)
