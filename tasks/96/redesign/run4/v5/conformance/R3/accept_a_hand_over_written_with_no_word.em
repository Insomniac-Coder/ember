#$ test: run-pass
#$ rules: OWN-8, OWN-1
#$ stdout: 2
## design rules: R3, R1
## R3, R1: the nearest programs that are allowed: `b = a` and `return xs` hand over with no word.

fn make() -> Array[int]:
    xs = [1, 2]
    return xs

fn main():
    a = make()
    b = a
    println(b.len())
