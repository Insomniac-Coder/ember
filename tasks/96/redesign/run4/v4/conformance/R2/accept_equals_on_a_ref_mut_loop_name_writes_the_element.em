#$ test: run-pass
#$ rules: R2
#$ stdout: 0 0
## R2: the nearest program that is allowed: `=` on a `ref mut` loop name writes the element it
## names.

fn main():
    xs = [1, 2]
    for ref mut x in xs:
        x = 0
    println(xs[0], xs[1])
