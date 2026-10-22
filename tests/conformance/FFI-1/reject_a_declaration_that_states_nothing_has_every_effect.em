#$ test: compile-fail
#$ rules: FFI-1, EFF-3
#$ error[E4001]: `@noalloc` function `count` reaches an allocation
#$ error[E4070]: `@deterministic` function `count` reaches a nondeterministic operation
# `[FFI-1]` — the effects of a function whose C declaration states none are unknown: every
# effect (`[EFF-3]`).

import c "./maths.c" as m

@noalloc @deterministic
fn count() -> i32:
    unsafe:  # SAFETY(ffi): `counter` takes nothing
        return m.counter()

fn main():
    println(count())
