#$ test: compile-fail
#$ rules: MOD-2
# Reading a private field from another module is `E1052`, and writing one is
# the same error for the same reason: the check is on resolution, so it does
# not matter which side of the `=` the field is on.

from support.access import make

fn main():
    p = make(1, 2, 3, 4)
    p.secret = 5          #$ error[E1052]: `secret` is private to `support.access.Panel`'s module
    println(p.w)
