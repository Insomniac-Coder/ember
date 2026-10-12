#$ test: compile-fail
#$ rules: MOD-8, MOD-2
## `[MOD-8]` — a glob binds the `pub` items only.

from support.shapes import *

fn main():
    println(hidden())    #$ error[E1010]: cannot find `hidden` in this scope
