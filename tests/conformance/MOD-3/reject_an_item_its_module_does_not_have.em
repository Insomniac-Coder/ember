#$ test: compile-fail
#$ rules: MOD-3
# `[MOD-3]` — an item missing from an existing module is `E1061` at the import.

from support.geometry import Nowhere    #$ error[E1061]: module 'support.geometry' has no item named `Nowhere`
from std.math import no_such_function    #$ error[E1061]: module 'std.math' has no item named `no_such_function`

fn main():
    println(1)
