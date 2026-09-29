#$ test: run-fail
#$ rules: OPT-2, TYP-8, PHIL-5
#$ profiles: debug, release, shipping
#$ stdout: 3
#$ panics: division by zero
# For MSVC the nest below runs in a function of its own (ADR-079); a check
# that fails in it still stops the program at the operation that failed,
# with the same message, on every compiler. `d[7]` is 2, so round 2 divides
# by zero there.

fn main():
    n = 100
    a: Array[int] = []
    d: Array[int] = []
    out: Array[int] = []
    for i in 0..n:
        a.push(i)
        d.push(10)
        out.push(0)
    d[7] = 2
    println(3)
    for round in 0..5:
        for i in 0..n:
            out[i] = a[i] // (d[i] - round)
    println(out[0])
