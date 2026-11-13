#$ test: compile-fail
#$ rules: GRM-38, GRM-29
## `[GRM-38]` — `(e,)` is `E0100` (`[GRM-29]`), not a generator and not a one-element tuple.

fn main():
    x = 3
    t = (x,)    #$ error[E0100]: a tuple has two or more elements; write `e`
    println(t)
