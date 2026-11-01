#$ test: compile-fail
#$ rules: FN-2a, GRM-15
# `[FN-2a]` (verdict 146) — `owns` before an argument for a parameter that only reads is `E3028`: the
# function does not take the list over.
#$ help: remove `owns`: `show(xs)`

fn show(xs: Array[int]):
    println(xs)

fn main():
    xs = [1, 2]
    show(owns xs)    #$ error[E3028]: `show` only reads its argument
