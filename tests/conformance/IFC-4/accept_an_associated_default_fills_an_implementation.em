#$ test: run-pass
#$ rules: IFC-4
#$ stdout: 3 7 4 40
# `[IFC-4]` (ODR-049, SP-003) — `type Out = Self` in an interface is what an
# implementation that does not state `Out` takes, read with `Self` as the
# implementing type; one that states it overrides it. A default may name
# another associated type (`type Key = Item`).

interface Combine:
    type Out = Self
    fn combine(self, other: Self) -> Out

struct Meters:
    v: int

extend Meters implements Combine:
    fn combine(self, other: Meters) -> Meters:
        return Meters(self.v + other.v)

struct Pair:
    v: int

extend Pair implements Combine:
    type Out = int
    fn combine(self, other: Pair) -> int:
        return self.v + other.v

interface Source:
    type Item
    type Key = Item
    fn first(self) -> Item
    fn key(self) -> Key

struct Nums:
    n: int

extend Nums implements Source:
    type Item = int
    fn first(self) -> int:
        return self.n
    fn key(self) -> int:
        return self.n * 10

fn both[T: Combine](a: T, b: T) -> T.Out:
    return a.combine(b)

fn keyed[S: Source[Key = int]](s: S) -> int:
    return s.key()

fn main():
    println(both(Meters(1), Meters(2)).v, both(Pair(3), Pair(4)), Nums(4).first(), keyed(Nums(4)))
