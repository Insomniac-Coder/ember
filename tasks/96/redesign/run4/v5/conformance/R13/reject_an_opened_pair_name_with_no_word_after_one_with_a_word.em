#$ test: compile-fail
#$ rules: GRM-42
#$ help: say what `b` is: `if pair exists as (ref mut a, ref b):`, or `ref mut b` to change it too
## design rules: R13, R3
## R13, R3: the names after `exists as` are a list of names without types, so a name with no word may
## not follow a name with a word (E0116).

fn main():
    pair: Option[(int, int)] = Some((1, 2))
    if pair exists as (ref mut a, b):  #$ error[E0116]: `ref mut` here belongs to `a` only
        a += b
