#$ test: compile-fail
#$ rules: EXP-5
#$ error[E2140]: assignment target is not a place
#$ help: assign the elements one at a time
# `[EXP-5]` — only a place can be assigned. A slice is a view, so writing to
# one used to fill a temporary and drop it, leaving the array unchanged.

fn main():
    xs = [1, 2, 3]
    xs[0..2] = [5, 6]
    println(xs)
