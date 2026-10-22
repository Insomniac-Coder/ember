#$ test: compile-fail
#$ rules: DIA-5, FFI-2
#$ error[E5002]: declared in the header at labels.h:8: `int label_sum(const int *items, size_t n)`
# `[DIA-5]`: an error about a declaration an import wrote from a header (here the overlay asserts
# `label_sum` safe while its pointer has no facts) names the header and the C declaration it
# came from, as the header writes it (its two lines joined) and where.

import c "./labels.h" with (overlay = "overlays/unstated.em") as labels

fn main():
    pass
