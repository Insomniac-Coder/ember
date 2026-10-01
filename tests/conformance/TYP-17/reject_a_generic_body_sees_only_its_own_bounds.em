#$ test: compile-fail
#$ rules: TYP-17, TYP-16
# D-433 — a generic body has only what its own bounds provide. `fn f[T]`
# saw `dup`, which `extend[T: Copy] Wrap[T]` gives only to a `Wrap` of a
# `Copy` type: its `Wrap[T]` was the same type as the one the extension's
# own methods were checked with, `T: Copy` in scope. Named `Q`, it was
# refused; named `T`, it was accepted and failed only at an instance.

struct Wrap[T]:
    x: T

extend[T: Copy] Wrap[T]:
    fn dup(self) -> int:
        return 1

fn f[T](w: Wrap[T]) -> int:
    return w.dup()    #$ error[E1010]: `Wrap[T]` has no method named `dup`

fn main():
    println(f(Wrap(x = "s".to_string())))
