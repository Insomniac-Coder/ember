#$ test: compile-fail
#$ rules: FN-5
# `[FN-5]` — a default may read an earlier parameter and neither changes it (`E3023`, as a plain
# name) nor hands it over.

fn last(xs: Array[int], n: Option[int] = xs.pop()):    #$ error[E3023]
    println(n)

fn main():
    last([1])
