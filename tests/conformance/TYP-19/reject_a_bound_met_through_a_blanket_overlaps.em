#$ test: compile-fail
#$ rules: TYP-19, GRM-34
# D-410 — a bound met only through a blanket implementation is met: a
# `Holder[int]` is a `Fetch[Gizmo]` (a `Gizmo` is a `Key[int]`), so
# `Crate[Holder[int]]` is a type both implementations below apply to. The
# overlap was missed.

interface Key[T]:
    fn key() -> T

interface Fetch[Q]:
    fn fetch(q: Q) -> int

interface Mark:
    fn mark() -> int

struct Gizmo:
    n: int

extend Gizmo implements Key[int]:
    fn key() -> int:
        return this.n

[T]
struct Holder:
    x: T

extend[T, Q: needs Key[T]] Holder[T] implements Fetch[Q]:
    fn fetch(q: Q) -> int:
        return 1

[U]
struct Crate:
    u: U

extend[U: needs Fetch[Gizmo]] Crate[U] implements Mark:
    fn mark() -> int:
        return 1

extend Crate[Holder[int]] implements Mark:   #$ error[E2041]: `Crate[Holder[i64]]` and `Crate[U]` can be one type, and both implement `Mark`
    fn mark() -> int:
        return 2

fn main():
    println(Crate(u = Holder(x = 1)).mark())
