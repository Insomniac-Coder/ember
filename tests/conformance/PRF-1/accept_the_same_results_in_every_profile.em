#$ test: run-pass
#$ rules: PRF-1, PRF-4
#$ profiles: debug, development, release
#$ stdout: 55 2 true
# `[PRF-1]` — a profile never changes what a program means: the same program computes the same
# results in every profile, and a `debug_assert` whose argument holds changes nothing.

fn main():
    xs: Array[int] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    total = 0
    for x in xs:
        total += x
    debug_assert(total == 55)
    println(total, xs[1], total // xs[0] == 55)
