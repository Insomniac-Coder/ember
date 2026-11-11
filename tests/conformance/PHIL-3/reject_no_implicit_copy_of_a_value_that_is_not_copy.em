#$ test: compile-fail
#$ rules: PHIL-3, OWN-1
#$ error[E3040]: `a` was handed over to `b` on line 9 and is used on line 10; to keep both, write `b = copy of a` (a copy of your own) or `b = shares a` (two owners of one value)
# `[PHIL-3]` — no implicit copy of a non-`Copy` value occurs: `b = a` moves the array, so `a`
# holds nothing afterwards. A copy is written: `a.clone()`.

fn main():
    a = [1, 2, 3]
    b = a
    println(b.len(), a.len())
