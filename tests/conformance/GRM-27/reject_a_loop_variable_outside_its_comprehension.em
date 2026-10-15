#$ test: compile-fail
#$ rules: GRM-27
#$ error[E1010]: cannot find `k` in this scope
# `[GRM-27]` — the loop variables are scoped to the comprehension.

fn main():
    xs = [1, 2]
    s = {k for k in xs}
    println(s, k)
