#$ test: compile-fail
#$ rules: MOD-8
## `[MOD-8]` — a name two glob imports bind to different items, once used, is
## `E1031`, naming both sources. Names only one glob binds stay usable.

from support.shapes import *
from support.words import *

fn main():
    println(area(Square(2)))
    println(describe())    #$ error[E1031]: `describe` is bound by two glob imports, from `support.shapes` and `support.words`
