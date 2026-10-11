#$ test: compile-fail
#$ rules: DSJ-7, UNS-1

# `[DSJ-7]` — an unchecked assumption is the caller's to answer for: outside
# `unsafe` it is `E3100`.

from std.mem import assume_disjoint

fn main():
    x: Array[int] = [1]
    (u, v) = assume_disjoint(x[..], x[..])    #$ error[E3100]: `assume_disjoint` requires an `unsafe` block
