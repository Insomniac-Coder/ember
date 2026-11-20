# Clause FFI-41.1: "A static data member, and a variable of a namespace, imports as a foreign static
# (`[FFI-10]`), read and written as a global (...), in `unsafe` unless the header declares it `const`
# and its type is `ThreadShared` (`[STA-1]`), holds no pointer and has no `mutable` member"; "a read
# outside `unsafe` of one that fails a condition is `E3100`"
#$ test: compile-fail
#$ rules: FFI-41, FFI-40a, STA-1
# `PLAIN` meets every condition and is read in safe code; `STATS`'s type has a `mutable` member: the
# spec's message, with this file's line number. The test only compiles.

import cpp "stats.hpp"

fn main():
    plain = cpp.PLAIN.total
    total = cpp.STATS.total    #$ error[E3100]: `STATS` is a C++ `const` object whose type `Stats` has a `mutable` member (`hits`), which C++ may change while this reads it; put line 14 in an `unsafe:` block with a `# SAFETY:` note
    println(plain, total)
