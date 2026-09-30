#$ test: run-pass
#$ rules: OPT-2, CTL-3b
#$ profiles: debug, release, shipping
#$ stdout:
#$ 28
#$ 247503
#$ 2.997
#$ 2970 6000
#$ 100000
#$ 49500
# For MSVC, a loop nest that carries one running value and writes no memory
# runs in a function of its own that takes its views as `restrict` pointers
# and returns the value (ADR-084): MSVC took no running value over two views
# as a reduction in a function where a list's header was passed to `push`.
# The first three nests move: over two views, over one view with the value
# coming in from before it, and a decimal one. Two running values, a nest
# that also writes a list, and a single loop stay. The results are the same
# on every compiler.

fn main():
    xs: Array[int] = []
    ys: Array[int] = []
    for i in 0..1000:
        xs.push(i % 100)
        ys.push(i % 7)

    total = 0
    for round in 0..20:
        for x in xs.iter().chain(ys.iter()):
            total ^= x + round
    println(total)

    sum = total % 5
    for round in 0..5:
        for x in xs.iter():
            sum += x
    println(sum)

    mean = 0.0
    for round in 0..4:
        for y in ys.iter().copied():
            mean += y as float
    println(mean / 4000.0)

    lo = 0
    hi = 0
    for round in 0..3:
        for x in xs.iter():
            lo += x % 3
            hi += x % 5
    println(f"{lo} {hi}")

    zs: Array[int] = []
    for i in 0..1000:
        zs.push(0)
    count = 0
    for round in 0..2:
        for i in 0..1000:
            zs[i] = xs[i] + round
            count += zs[i]
    println(count)

    single = 0
    for x in xs.iter():
        single += x
    println(single)
