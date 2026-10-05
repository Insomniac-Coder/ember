#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 0
# D-517 — an associated type's bound is checked once every implementation is known. An instance
# a generic extension's signature names (`Boxy[T]` in `Holder[T].boxed`) is made while the
# declarations are still read; its `Size`, defaulted to `int`, was checked against `Counted`
# then, before `extend int implements Counted` below was read, and reported as unmet.

interface Counted:
    fn of(n: int) -> Self

interface Seq:
    type Size: Counted = int
    fn next(mut self) -> Option[int]

struct Boxy[T]:
    x: T

extend[T] Boxy[T] implements Seq:
    fn next(mut self) -> Option[int]:
        return None

struct Holder[T]:
    x: T

extend[T: Copy] Holder[T]:
    fn boxed(self) -> Boxy[T]:
        return Boxy(self.x)

extend int implements Counted:
    fn of(n: int) -> int:
        return n

fn main():
    h = Holder(5)
    b = h.boxed()
    println(int.of(b.x - 5))
