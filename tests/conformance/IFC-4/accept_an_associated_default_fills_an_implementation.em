#$ test: run-pass
#$ rules: IFC-4
#$ stdout: 3 7 4 40
# `[IFC-4]` (ODR-049, SP-003) — `type Out = Self` in an interface is what an
# implementation that does not state `Out` takes, read with `Self` as the
# implementing type; one that states it overrides it. A default may name
# another associated type (`type Key = Item`).

interface Combine:
    type Out = Self
    fn combine(other: Self) -> Out

struct Meters:
    v: int

extend Meters implements Combine:
    fn combine(other: Meters) -> Meters:
        return Meters(this.v + other.v)

struct Pair:
    v: int

extend Pair implements Combine:
    type Out = int
    fn combine(other: Pair) -> int:
        return this.v + other.v

interface Source:
    type Item
    type Key = Item
    fn first() -> Item
    fn key() -> Key

struct Nums:
    n: int

extend Nums implements Source:
    type Item = int
    fn first() -> int:
        return this.n
    fn key() -> int:
        return this.n * 10

[T: needs Combine]
fn both(a: T, b: T) -> T.Out:
    return a.combine(b)

[S: needs Source[Key = int]]
fn keyed(s: S) -> int:
    return s.key()

fn main():
    println(both(Meters(1), Meters(2)).v, both(Pair(3), Pair(4)), Nums(4).first(), keyed(Nums(4)))
