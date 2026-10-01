#$ test: compile-fail
#$ rules: TYP-19, TYP-20
# D-409 — an implementation over type parameters whose interface the overlap
# check could not read was left out of it, so its overlap reached C as one
# function defined twice, with no error: an interface argument defaulting to
# `Self` on a built-in generic type, and one naming an associated type.

interface Combine[R = Self]:
    fn combine(self, other: R) -> int

interface Conv[T]:
    fn conv(self) -> T

struct Counter:
    n: int

extend Counter implements Iterator:
    type Item = int

    fn next(mut self) -> Option[int]:
        return None

struct Wrap[T]:
    x: T

extend[T] Array[T] implements Combine:
    fn combine(self, other: Array[T]) -> int:
        return self.len() + other.len()

extend Array[int] implements Combine:   #$ error[E2041]: `Array[i64]` and `Array[T]` can be one type, and both implement `Combine[Array[i64]]`
    fn combine(self, other: Array[int]) -> int:
        return 0

extend[I: Iterator] Wrap[I] implements Conv[I.Item]:
    fn conv(self) -> I.Item:
        return todo()

extend[U] Wrap[U] implements Conv[int]:   #$ error[E2041]: `Wrap[U]` and `Wrap[I]` can be one type, and both implement `Conv[i64]`
    fn conv(self) -> int:
        return 1

interface Tag[T]:
    fn tag(self) -> int

# Only the `implements` names `I.Item`; no method does.
extend[I: Iterator] Wrap[I] implements Tag[I.Item]:
    fn tag(self) -> int:
        return 1

extend[U] Wrap[U] implements Tag[int]:   #$ error[E2041]: `Wrap[U]` and `Wrap[I]` can be one type, and both implement `Tag[i64]`
    fn tag(self) -> int:
        return 2

fn use_it[C: Combine](c: C, d: C) -> int:
    return c.combine(d)

fn main():
    println(use_it([1, 2], [3]))
    w = Wrap(Counter(n = 0))
    println(w.conv())
