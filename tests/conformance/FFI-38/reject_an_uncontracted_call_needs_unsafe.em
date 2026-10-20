#$ test: compile-fail
#$ rules: FFI-38, FFI-2
#$ help: `fn uncontracted_sum(items: TODO(ownership) TODO(count) TODO(nullable), count)` in an `unsafe overlay c "uncontracted.h":`
# `[FFI-38]` — the importer never guesses a fact the header does not state: a pointer's ownership,
# count and nullability, and whether a function is safe to call at all (`[FFI-2]`: never derived,
# since a scalar can be an address). Every imported function is uncontracted, so each call is
# `E5002` outside `unsafe`, naming the unknown facts and the overlay line that supplies them.

import c "uncontracted.h"

fn main():
    xs: [i32; 3] = [1, 2, 3]
    println(c.uncontracted_add(1, 2))  #$ error[E5002]: a foreign call without a complete safe contract requires `unsafe`
    println(c.uncontracted_sum(ref_to_ptr(ref xs[0]), 3))  #$ error[E5002]: a foreign call without a complete safe contract requires `unsafe`
