#$ test: compile-fail
#$ rules: STD-12, STD-17, TYP-17
#$ help: `m.insert(k, v)` takes the key itself
# `[STD-12]` (ODR-067) — `m[k] = v` makes the key from `k` when the key is
# new (`ToKey`), and for a `K` that is its own lookup type that is a clone.
# A key type that is not `Clone` is `E2040`, and the help names `insert`,
# which takes the key itself.

from std.collections import Hash, Hasher

@no_derive(Clone)
struct Ticket:
    n: int

extend Ticket implements Eq, Hash:
    fn eq(self, other: Ticket) -> bool:
        return self.n == other.n

    fn hash[H: Hasher](self, mut h: H):
        h.write_i64(self.n)

fn main():
    m: Map[Ticket, int] = {}
    m[Ticket(1)] = 5 #$ error[E2040]: `Ticket` does not implement `std.collections.ToKey[Ticket]`, which `Q` requires
    m.insert(Ticket(2), 6)
    println(m.len())
