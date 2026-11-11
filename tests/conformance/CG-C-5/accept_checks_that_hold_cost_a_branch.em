#$ test: run-pass
#$ rules: CG-C-5, TYP-8
#$ profiles: debug, development
#$ stdout: 3 3
#$ assert-c: contains("ember_panic_bounds(")
# `[CG-C-5]` — a check that holds falls through its forward branch: the panic call is on the
# failing side only, so the program runs on and prints.

fn main():
    xs: Array[int] = [1, 2, 3]
    i = xs.len() - 1
    println(xs[i], xs[0] + xs[1])
