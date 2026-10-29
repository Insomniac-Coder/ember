#$ test: run-pass
#$ rules: FFI-1, EFF-3, DET-2
#$ warning[L3015]: `square` is a C function the program calls in unsafe code
#$ warning[L3015]: `offset` is a C function the program calls in unsafe code
#$ warning[L3015]: `counter` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 49 5
#$ stdout: 1
# `[FFI-1]` — a foreign declaration's effects are derived where the C declaration gives them:
# `__attribute__((const))` has none and the same result for the same arguments, so a
# `@noalloc … @deterministic` caller may call it; `__attribute__((pure))` has none (it only reads
# memory). A function whose declaration states nothing has every effect (`[EFF-3]`) and an unknown
# thread rule. The facts make no call safe: these are still `unsafe` to call.

import c "./maths.c" as m

@noalloc @nolock @noblock @noio @nosync @nopanic(explicit) @deterministic
fn squared(x: i32) -> i32:
    unsafe:  # SAFETY(ffi): `square` takes any integer
        return m.square(x)

@noalloc @nolock @noblock @noio @nosync
fn shifted(index: i32) -> i32:
    unsafe:  # SAFETY(ffi): `offset` takes any index
        return m.offset(index)

fn main():
    println(squared(7), shifted(2))
    unsafe:  # SAFETY(ffi): `counter` takes nothing
        println(m.counter())
