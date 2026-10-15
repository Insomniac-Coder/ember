#$ test: compile-fail
#$ rules: PHIL-3, OWN-1
#$ error[E3040]: `a` has been moved out of
# `[PHIL-3]` — no implicit copy of a non-`Copy` value occurs: `b = a` moves the array, so `a`
# holds nothing afterwards. A copy is written: `a.clone()`.

fn main():
    a = [1, 2, 3]
    b = a
    println(b.len(), a.len())
