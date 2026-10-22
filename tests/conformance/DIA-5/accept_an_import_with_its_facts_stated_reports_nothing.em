#$ test: run-pass
#$ rules: DIA-5
#$ stdout: 5
#$ 8
# `[DIA-5]`'s other side: the C declaration is named only by an error about it. With the facts
# the calls need stated in the overlay, the same calls compile with nothing reported (the header's
# functions defined in `labels.c`, which the case imports so that it links and runs).

import c "./labels.c" with (overlay = "overlays/stated.em") as labels

fn main():
    println(labels.label_length(c"hello"))
    println(labels.label_twice(4))
