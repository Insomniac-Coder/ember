#$ test: run-pass
#$ rules: HASH-2, STD-11, HASH-1
#$ stdout: k0 k1 k2 k3 k4 k5 k6 k7 k8 k9
#$ stdout: 45 true true true
# `[HASH-2]` — `RandomState` is the per-process randomly seeded hasher for
# maps keyed by untrusted input: a `Map[K, V, RandomState]` works as any map
# and still iterates in insertion order (`[STD-11]`), so the seed is never
# seen; within one process a key hashes one way. It did not exist.

from std.collections import RandomState

fn seeded(s: str) -> u64:
    h = RandomState.new()
    s.hash(h)
    return h.finish()

fn main():
    m: Map[String, int, RandomState] = {}
    for i in range(10):
        m[f"k{i}"] = i
    keys: Array[String] = []
    total = 0
    for (k, v) in m.items():
        keys.push(k.clone())
        total += v
    println(keys.join(" "))
    println(total, m["k3"] == 3, seeded("key") == seeded("key"), seeded("key") != seeded("kez"))
