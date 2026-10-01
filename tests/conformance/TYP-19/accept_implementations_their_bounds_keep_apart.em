#$ test: run-pass
#$ rules: TYP-19
#$ stdout: 2 2 3
# D-410 (ODR-092) — a bound separates two implementations where the type it
# bounds is written out in full and does not meet it, as the unification
# makes it: `U: Conv[T]` at `T = int` asks `Conv[int]` of `Gadget`; an
# associated-type binding (`Iterator[Item = int]`) is part of the bound; and
# `Wrap[V]` with `V` made `int` is `Wrap[int]`, written out in full. Each
# pair was `E2041`.

interface Conv[T]:
    fn conv(self) -> T

interface Mark:
    fn mark(self) -> int

interface Tagged:
    fn tag(self) -> int

interface Foo:
    fn foo(self) -> int

struct Gadget:
    n: int

extend Gadget implements Conv[bool]:
    fn conv(self) -> bool:
        return true

struct Pair[A, B]:
    a: A
    b: B

extend[T, U: Conv[T]] Pair[T, U] implements Mark:
    fn mark(self) -> int:
        return 1

extend Pair[int, Gadget] implements Mark:
    fn mark(self) -> int:
        return 2

struct Words:
    n: int

extend Words implements Iterator:
    type Item = str
    fn next(mut self) -> Option[str]:
        return None

struct Holder[T]:
    x: T

extend[T: Iterator[Item = int]] Holder[T] implements Tagged:
    fn tag(self) -> int:
        return 1

extend Holder[Words] implements Tagged:
    fn tag(self) -> int:
        return 2

struct Wrap[T]:
    x: T

struct Two[A, B]:
    a: A
    b: B

extend[T: Foo] Two[T, int] implements Mark:
    fn mark(self) -> int:
        return 1

extend[V] Two[Wrap[V], V] implements Mark:
    fn mark(self) -> int:
        return 3

fn main():
    p = Pair(a = 1, b = Gadget(n = 0))
    h = Holder(x = Words(n = 0))
    t = Two(a = Wrap(x = 1), b = 2)
    println(p.mark(), h.tag(), t.mark())
