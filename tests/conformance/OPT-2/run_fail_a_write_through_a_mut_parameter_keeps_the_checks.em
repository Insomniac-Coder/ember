#$ test: run-fail
#$ rules: OPT-2
#$ profiles: debug, release, shipping
#$ panics: index 1 is out of bounds for a length of 0
# The loop clears its own `mut` parameter: it keeps every check and panics on
# the second pass.

fn sum_and_empty(mut xs: Array[int]) -> int:
    total = 0
    for i in 0..4:
        total = total + xs[i]
        xs.clear()
    return total

fn main():
    xs: Array[int] = [1, 2, 3, 4]
    println(sum_and_empty(xs))
