# Clause BRW-13.6: "A name written **`const ref mut`** is never given another ref, as `const` on a
# value means it never changes: `r = const ref mut a` then `r = ref mut b` is `E3023` ("`r` is `const
# ref mut a`, so it always refers to `a`; to refer to `b` use a new name, `rb = ref mut b`, or make
# `r` with plain `ref mut`: `r = ref mut a`")."
#$ test: compile-fail
#$ rules: BRW-13, TYP-14
# The spec's message.

fn main():
    a = 1
    b = 2
    r = const ref mut a
    r = 20
    r = ref mut b    #$ error[E3023]: `r` is `const ref mut a`, so it always refers to `a`; to refer to `b` use a new name, `rb = ref mut b`, or make `r` with plain `ref mut`: `r = ref mut a`
    println(a, b)
