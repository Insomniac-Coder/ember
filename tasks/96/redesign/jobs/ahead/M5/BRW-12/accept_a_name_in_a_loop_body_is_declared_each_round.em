# Clause BRW-12.1: "A name is declared by the line that first gives it a value; a line inside a
# loop's body declares it once per round (`[GRM-4]`)."
#$ test: run-pass
#$ rules: BRW-12, GRM-4, BRW-13
#$ stdout: 11 12 13
#$ stdout: 36
# Each round's `r` is a new name, so `r = const ref mut xs[i]` never gives the last round's `r`
# another ref (that would be `E3023`, `[BRW-13]`). `total`, in scope before the loop, is assigned.

fn main():
    xs = [1, 2, 3]
    for i in 0..3:
        r = const ref mut xs[i]
        r += 10
    println(xs[0], xs[1], xs[2])
    total = 0
    for x in xs:
        total += x
    println(total)
