#$ test: compile-fail
#$ rules: TYP-19, GRM-34
# D-410 — a bound met only through a blanket implementation is met: a
# `Holder[int]` is a `Fetch[Gizmo]` (a `Gizmo` is a `Key[int]`), so
# `Crate[Holder[int]]` is a type both implementations below apply to. The
# overlap was missed.

interface Key[T]:
    fn key(self) -> T

interface Fetch[Q]:
    fn fetch(self, q: Q) -> int

interface Mark:
    fn mark(self) -> int

struct Gizmo:
    n: int

extend Gizmo implements Key[int]:
    fn key(self) -> int:
        return self.n

struct Holder[T]:
    x: T

extend[T, Q: Key[T]] Holder[T] implements Fetch[Q]:
    fn fetch(self, q: Q) -> int:
        return 1

struct Crate[U]:
    u: U

extend[U: Fetch[Gizmo]] Crate[U] implements Mark:
    fn mark(self) -> int:
        return 1

extend Crate[Holder[int]] implements Mark:   #$ error[E2041]: `Crate[Holder[i64]]` and `Crate[U]` can be one type, and both implement `Mark`
    fn mark(self) -> int:
        return 2

fn main():
    println(Crate(u = Holder(x = 1)).mark())
