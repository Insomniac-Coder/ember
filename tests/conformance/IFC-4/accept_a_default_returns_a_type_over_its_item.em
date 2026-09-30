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
    fn get(mut self) -> Item

    fn wrapped(owned self) -> Wrap[Self, Item]:
        return Wrap(self)

struct Wrap[S, T]:
    s: S

extend[T, S: Source[Item = T]] Wrap[S, T]:
    fn first(mut self) -> T:
        return self.s.get()

struct Ints implements Source:
    n: int
    type Item = int

    fn get(mut self) -> int:
        return self.n

fn main():
    w = Ints(n = 3).wrapped()
    println(w.first())
