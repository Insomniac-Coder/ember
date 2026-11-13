#$ test: compile-fail
#$ rules: GRM-42
## `[GRM-42]` — in a list of names declared without types of their own (a loop's names, a pattern's names,
## the names after `exists as`) each word belongs to the one name after it, and a name with no word may
## not follow a name with a word (`E0116`), so that no word can be read as covering the next name.

fn run(f: fn(ref mut int, int)):
    n = 1
    f(ref mut n, 2)

fn main():
    pairs = [(1, 2)]
    for ref mut pos, vel in pairs:    #$ error[E0116]: `ref mut` here belongs to `pos` only; say what `vel` is
                                      #$ help: write `ref vel` to read it, or `ref mut vel` to treat it as `pos` is treated
        println(pos)
    pair: Option[(int, int)] = Some((1, 2))
    if pair exists as (ref mut a, b):    #$ error[E0116]: `ref mut` here belongs to `a` only; say what `b` is
        println(a)
    run(fn(ref mut x, y): x += y)    #$ error[E0116]: `ref mut` here belongs to `x` only; say what `y` is
