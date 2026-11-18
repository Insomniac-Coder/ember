#$ test: run-pass
#$ rules: STR-7
#$ stdout: 2 1 1 0 7
#$ 1 x
## `[STR-7]` - inside the body of a `struct`, `enum`, `class` or `extend` block, `Self` names the type
## being declared with its own generic parameters, and the type's name may be written with any
## arguments: a method may return `Self`, `Pair[T]` or `Pair[int]`.

[T]
struct Pair:
    a: T
    b: T

    fn swap() -> Self:
        return Self(this.b, this.a)

    fn as_pair() -> Pair[T]:
        return Pair[T](this.a, this.b)

    fn as_int() -> Pair[int]:
        return Pair[int](1, 2)

    static fn make(a: T) -> Self:
        return Self(a, a)

enum Opt[T]:
    Has(T)
    Nope

    fn me() -> Self:
        return Opt[T].Nope

    fn other() -> Opt[int]:
        return Opt[int].Has(1)

[T]
class Node:
    v: T

    fn again() -> Self:
        return Self(this.v)

    fn other() -> Node[StrView]:
        return Node[StrView]("x")

extend[T] Pair[T]:
    fn dup() -> Self:
        return Pair[T](this.a, this.a)

    fn nums() -> Pair[int]:
        return Pair[int](0, 0)

fn main():
    p = Pair(1, 2)
    q = p.swap()
    println(q.a, p.as_int().a, p.dup().b, p.nums().a, Pair.make(7).b)
    o = Opt[int].Has(2)
    _m = o.me()
    _n = o.other()
    println(Node(1).again().v, Node(1).other().v)
