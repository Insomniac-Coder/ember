#$ test: compile-fail
#$ rules: FN-6, MOD-2
# `[FN-6]`, `[MOD-2]` — a function reached through its module as a value is visible as a call of it
# would be: a private one is not.

import items.steps

fn main():
    f = steps.hidden  #$ error[E1052]: `hidden` is private to `items.steps`
    println(f(1))
