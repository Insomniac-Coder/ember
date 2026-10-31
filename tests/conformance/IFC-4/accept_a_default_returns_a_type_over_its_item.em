#$ test: run-pass
#$ rules: IFC-4, STD-19, TYP-16
#$ stdout: 3
# D-407 (1) — a default method returning a generic type over its interface's
# own associated type (`Wrap[Self, Item]`): comparing the implementation's
# signature made `Wrap[Ints, Self.Item]` before `Item` was read as `Ints`'s,
# and the extension's bodies were checked for it (`E2020` expected
# `Self.Item`, found `i64`). An instance over an associated type not yet read
# is now as opaque as one over a parameter: signatures only.

interface Source:
    type Item
    fn get() -> Item

    fn wrapped() owns this -> Wrap[Self, Item]:
        return Wrap(this)

[S, T]
struct Wrap:
    s: S

extend[T, S: needs Source[Item = T]] Wrap[S, T]:
    fn first() -> T:
        return this.s.get()

struct Ints implements Source:
    n: int
    type Item = int

    fn get() -> int:
        return this.n

fn main():
    w = Ints(n = 3).wrapped()
    println(w.first())
