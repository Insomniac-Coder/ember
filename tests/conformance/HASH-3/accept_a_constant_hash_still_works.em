#$ test: run-pass
#$ rules: HASH-3, HASH-4, STD-11
#$ stdout: 100 Some(42) None
# `[HASH-3]` — a key whose `hash` gives every key the same value is slow but
# never wrong: every lookup still ends, and finds exactly its own key.

from std.collections import Hash, Hasher

struct Id:
    n: int

extend Id implements Hash:
    fn hash[H: Hasher](self, mut h: H):
        h.write_u8(7)

fn main():
    m: Map[Id, int] = {}
    for i in range(100):
        m[Id(i)] = i
    println(len(m), m.get(Id(42)), m.get(Id(100)))
