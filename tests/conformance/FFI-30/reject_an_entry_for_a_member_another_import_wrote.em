#$ test: compile-fail
#$ rules: FFI-30, FFI-30c
#$ profiles: debug
# `[FFI-30]` (D-879) — a C++ class two imports of one header bind is one entity, its members written
# once, by the import that wrote the class: an entry of the second import's overlay for one of its
# members states nothing (`E5010`, naming the import whose entity it is), never a silent choice
# between two contracts (`[FFI-30c]`). Its entries for the header's functions are its own.

import cpp "classes.hpp" with (overlay = "overlays/classes.em")
import cpp "classes.hpp" with (overlay = "overlays/classes_counted.em") as counted    #$ error[E5010]: the overlay's entry for `Counter.count` states nothing

fn main():
    println(counted.doubled(2))
