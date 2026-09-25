#$ test: compile-fail
#$ rules: MOD-2, CT-1
# `[MOD-2]` — a type's `const` is private to its module unless it is `pub`,
# as a field is: `Grid.SIDE` reads anywhere, `Grid.SECRET` only through
# what its module exports.

from support.constants import Grid, secret

fn main():
    println(Grid.SIDE, secret())
    println(Grid.SECRET)    #$ error[E1052]: `SECRET` is private to
