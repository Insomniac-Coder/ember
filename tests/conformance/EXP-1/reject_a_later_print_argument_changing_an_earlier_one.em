#$ test: compile-fail
#$ rules: EXP-1, BRW-5, STD-9
# `[EXP-1]` — `println`'s arguments are evaluated left to right, where they
# are written, as a call's are: a place that is not `Copy` is borrowed there,
# so a later argument that changes it conflicts with the borrow (`E3021`), as
# `show(xs, xs.pop())` does.

fn main():
    xs = [1, 2]
    println(xs, xs.pop())  #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
