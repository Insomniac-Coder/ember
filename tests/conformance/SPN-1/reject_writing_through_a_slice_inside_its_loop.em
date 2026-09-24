#$ test: compile-fail
#$ rules: SPN-1, CTL-2
#$ error[E3021]: cannot write through a shared `Span`
# The same holds inside a loop over the array: the slice cannot be the way
# around `[CTL-2]`'s rule that the iterable stays unchanged.

fn main():
    xs = [1, 2, 3]
    total = 0
    for x in xs:
        xs[..][2] = 100
        total += x
    println(total)
