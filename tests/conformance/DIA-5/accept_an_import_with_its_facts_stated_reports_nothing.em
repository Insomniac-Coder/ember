#$ test: compile-pass
#$ rules: DIA-5
# `[DIA-5]`'s other side: the C declaration is named only by an error about it. With the facts
# the calls need stated in the overlay, the same import and calls compile with nothing reported.

import c "./labels.h" with (overlay = "overlays/stated.em") as labels

fn main():
    println(labels.label_length(c"hello"))
    println(labels.label_twice(4))
