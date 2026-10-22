#$ test: compile-fail
#$ rules: DIA-5, FFI-2, FFI-15
# `[DIA-5]`: an FFI error names the header and the C declaration, as the header writes it (on one
# line, a body elided) and where. A call with no safe contract (`E5002`) names the declaration it
# calls; a `str` passed where the declaration takes a NUL-terminated `const char *` (`E5020`) names
# the declaration of the parameter.

import c "./labels.h" with (overlay = "overlays/stated.em") as labels

fn main():
    items: [i32; 3] = [1, 2, 3]
    println(labels.label_sum(ref_to_ptr(ref items[0]), 3))    #$ error[E5002]: declared in the header at labels.h:8: `int label_sum(const int *items, size_t n)`
    text: str = "hello"
    println(labels.label_length(text))    #$ error[E5020]: declared in the header at labels.h:6: `int label_length(const char *text)`
    println(labels.label_twice(4))
