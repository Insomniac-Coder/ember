#$ test: run-pass
#$ rules: STD-12, STD-16
#$ stdout: Some(10) true 20
#$ stdout: Some(10) false 1
# `[STD-12]` (ODR-067, SP-016) — looking a key up needs only `AsKey`, which
# every `K: Eq + Hash` is, so a key type that cannot be cloned is looked up,
# inserted with `insert` (which moves the key in) and removed. Only
# `m[k] = v` for a new key needs `ToKey` (it makes the key from `k`).

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
    m.insert(Ticket(1), 10)
    m.insert(Ticket(2), 20)
    println(m.get(Ticket(1)), Ticket(2) in m, m[Ticket(2)])
    println(m.remove(Ticket(1)), Ticket(1) in m, m.len())
