#$ test: compile-fail
#$ rules: STD-12, STD-17, TYP-17
# `[STD-12]` (ODR-067) — a lookup type that implements only `AsKey[K]` looks
# keys up but cannot make one, so `m[q] = v`, which inserts when the key is
# new, is `E2040`; lookups through it are fine.

from std.collections import Hash, Hasher, AsKey

struct Id:
    n: int

extend Id implements Eq, Hash:
    fn eq(other: Id) -> bool:
        return this.n == other.n

    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_i64(this.n)

struct Probe:
    n: int

extend Probe implements Hash, AsKey[Id]:
    [H: needs Hasher]
    fn hash(ref mut h: H):
        h.write_i64(this.n)

    fn is_key(key: Id) -> bool:
        return this.n == key.n

fn main():
    m: Map[Id, int] = {}
    m.insert(Id(1), 7)
    println(m.get(Probe(1)))
    m[Probe(2)] = 8 #$ error[E2040]: `Probe` does not implement `std.collections.ToKey[Id]`, which `Q` requires
