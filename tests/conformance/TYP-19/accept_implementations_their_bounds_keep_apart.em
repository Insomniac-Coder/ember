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
    fn conv() -> T

interface Mark:
    fn mark() -> int

interface Tagged:
    fn tag() -> int

interface Foo:
    fn foo() -> int

struct Gadget:
    n: int

extend Gadget implements Conv[bool]:
    fn conv() -> bool:
        return true

[A, B]
struct Pair:
    a: A
    b: B

extend[T, U: needs Conv[T]] Pair[T, U] implements Mark:
    fn mark() -> int:
        return 1

extend Pair[int, Gadget] implements Mark:
    fn mark() -> int:
        return 2

struct Words:
    n: int

extend Words implements Iterator:
    type Item = StrView
    fn next() -> Option[StrView]:
        return None

[T]
struct Holder:
    x: T

extend[T: needs Iterator[Item = int]] Holder[T] implements Tagged:
    fn tag() -> int:
        return 1

extend Holder[Words] implements Tagged:
    fn tag() -> int:
        return 2

[T]
struct Wrap:
    x: T

[A, B]
struct Two:
    a: A
    b: B

extend[T: needs Foo] Two[T, int] implements Mark:
    fn mark() -> int:
        return 1

extend[V] Two[Wrap[V], V] implements Mark:
    fn mark() -> int:
        return 3

fn main():
    p = Pair(a = 1, b = Gadget(n = 0))
    h = Holder(x = Words(n = 0))
    t = Two(a = Wrap(x = 1), b = 2)
    println(p.mark(), h.tag(), t.mark())
