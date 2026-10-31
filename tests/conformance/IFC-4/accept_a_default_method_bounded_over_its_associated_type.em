#$ test: run-pass
#$ rules: IFC-4, TYP-17, TYP-23
#$ stdout: 3 2
# D-407 (5) — a default method may bound its own parameter over the
# interface's associated type (`C: Build[Item]`); an implementation reads
# the bound as its `Item` says (`Build[int]`), and the result's type at the
# call says what `C` is. It was `E2040` against the interface's own
# signature, then `E2060`.

interface Build[T]:
    static fn build_from(xs: Array[T]) -> Self

struct Bag:
    n: int

extend Bag implements Build[int]:
    static fn build_from(xs: Array[int]) -> Bag:
        return Bag(n = xs.len())

interface Src:
    type Item
    fn all() -> Array[Item]
    [C: needs Build[Item]]
    fn gather() -> C:
        return C.build_from(this.all())

struct S:
    xs: Array[int]

extend S implements Src:
    type Item = int
    fn all() -> Array[int]:
        return this.xs.clone()

fn main():
    b: Bag = S(xs = [1, 2, 3]).gather()
    c = S(xs = [4, 5]).gather[Bag]()
    println(b.n, c.n)
