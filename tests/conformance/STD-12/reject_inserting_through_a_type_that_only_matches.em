#$ test: compile-fail
#$ rules: STD-12, STD-17, TYP-17
# `[STD-12]` (ODR-067) — a lookup type that implements only `AsKey[K]` looks
# keys up but cannot make one, so `m[q] = v`, which inserts when the key is
# new, is `E2040`; lookups through it are fine.

from std.collections import Hash, Hasher, AsKey

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

fn main():
    m: Map[Id, int] = {}
    m.insert(Id(1), 7)
    println(m.get(Probe(1)))
    m[Probe(2)] = 8 #$ error[E2040]: `Probe` does not implement `std.collections.ToKey[Id]`, which `Q` requires
