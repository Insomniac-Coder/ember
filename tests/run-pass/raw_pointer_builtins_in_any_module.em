#$ test: run-pass
#$ profiles: debug
#$ stdout: true 7
# ODR-075, ODR-076 — `null[P]()` and `ref_to_ptr(r)` are the compiler's in every module, not only
# the root one (they were "cannot find" outside it).

import raw_pointer_builtins_support.pointers

fn main():
    value: i32 = 7
    println(pointers.none() == null[*mut i32](), pointers.first(value))
