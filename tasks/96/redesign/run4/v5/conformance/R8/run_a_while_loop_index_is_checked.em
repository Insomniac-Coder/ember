#$ test: run-fail
#$ rules: BRW-16
#$ stdout: 2
#$ panics: index 1 is out of bounds for a length of 1
## design rules: R8
## R8 (c): a `while` loop that may add and remove is not a walk, so each index is checked: a position
## past the shortened list stops the program with today's bounds message (section 2, bounds checks).

fn main():
    xs = [1, 2, 3]
    i = 0
    while i < xs.len():
        xs.remove(0)
        println(xs[i])
        i += 1
