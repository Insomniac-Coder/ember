#$ test: compile-fail
#$ rules: MOD-2, STR-1
# `pub(read)` is the asymmetric class: the read above is accepted and this
# write is not. Without it a module could only choose between hiding a field
# and surrendering its invariant.

from support.access import make

fn main():
    p = make(1, 2, 3, 4)
    println(p.title)
    p.title = 7           #$ error[E1050]: `support.access.Panel.title` is read-only outside its module
