#$ test: run-pass
#$ rules: R3, R1
#$ stdout: 2
## R3, R1: the nearest programs that are allowed: `b = a` and `return xs` hand over with no word.

fn make() -> Array[int]:
    xs = [1, 2]
    return xs

fn main():
    a = make()
    b = a
    println(b.len())
