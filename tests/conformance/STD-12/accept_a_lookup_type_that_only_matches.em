#$ test: run-pass
#$ rules: STD-12, STD-16, STD-17
#$ stdout: Some(7) true 7
#$ stdout: made 1
#$ stdout: {Id(n=1): 8, Id(n=2): 9}
#$ stdout: made 1
# `[STD-12]` (ODR-067, SP-016) — a lookup type implements `AsKey[K]` to be
# matched against keys (`is_key`, hashing as the key does), and `ToKey[K]`
# only if it can also make one. `Probe` only matches, so it looks keys up
# and cannot insert; `Maker` makes a key only when `m[q] = v` finds none, so
# replacing an existing entry makes nothing.

from std.collections import Hash, Hasher, AsKey, ToKey

struct Id:
    n: int

extend Id implements Eq, Hash:
    fn eq(self, other: Id) -> bool:
        return self.n == other.n

    fn hash[H: Hasher](self, mut h: H):
        h.write_i64(self.n)

struct Probe:
    n: int

extend Probe implements Hash, AsKey[Id]:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i64(self.n)

    fn is_key(self, key: Id) -> bool:
        return self.n == key.n

struct Maker:
    n: int
    made: Cell[int]

extend Maker implements Hash, AsKey[Id], ToKey[Id]:
    fn hash[H: Hasher](self, mut h: H):
        h.write_i64(self.n)

    fn is_key(self, key: Id) -> bool:
        return self.n == key.n

    fn to_key(self) -> Id:
        self.made.set(self.made.get() + 1)
        return Id(self.n)

fn main():
    m: Map[Id, int] = {}
    m.insert(Id(1), 7)
    println(m.get(Probe(1)), Probe(1) in m, m[Probe(1)])
    maker = Maker(2, Cell(0))
    m[maker] = 9
    println("made", maker.made.get())
    m[Maker(1, Cell(0))] = 8
    println(m)
    m[maker] = 9
    println("made", maker.made.get())
