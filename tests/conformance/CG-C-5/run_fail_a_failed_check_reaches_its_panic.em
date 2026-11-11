#$ test: run-fail
#$ rules: CG-C-5, TYP-8
#$ profiles: debug, development
#$ panics: index 3 is out of bounds for a length of 3
#$ assert-c: contains("ember_panic_bounds(")
#$ assert-c: contains("ember_panic_overflow(")
# `[CG-C-5]` — each check branches to a panic call that does not return: a failed bounds check
# stops the program with its message, and the emitted C calls the panic function only on the
# failing branch.

fn main():
    xs: Array[int] = [1, 2, 3]
    i = xs.len()
    total = xs[0] + xs[1]
    println(total, xs[i])
