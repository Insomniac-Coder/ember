#$ test: compile-fail
#$ rules: ERR-7, TYP-19
# `[ERR-7]` — a program's own `extend T implements From[T]` is `E2041`: the
# prelude's identity already gives it, to every type, an instance of a generic
# type and a type a blanket implementation reaches included (`[TYP-19]`).

struct B:
    v: int

extend B implements From[B]:    #$ error[E2041]: `B` already implements `From[B]`
    fn from(owned value: B) -> B:
        return value

struct Wrap[T]:
    v: T

extend[T] Wrap[T] implements From[Wrap[T]]:    #$ error[E2041]: `Wrap[T]` already implements `From[Wrap[T]]`
    fn from(owned value: Wrap[T]) -> Wrap[T]:
        return value

interface Named:
    fn name(self) -> int

struct C implements Named:
    v: int

    fn name(self) -> int:
        return self.v

extend[X: Named] C implements From[X]:    #$ error[E2041]: `C` already implements `From[X]`
    fn from(owned value: X) -> C:
        return C(value.name())

fn main():
    println(B.from(B(1)).v)
