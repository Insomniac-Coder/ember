#$ test: run-pass
#$ rules: R14, R5
#$ stdout: ok
## R14: "a value tested in its place stays a place, `if this.pools[k] is Pool[T] as ref mut pool:`
## names a view of the registry's pool that may change it (where the function may change the
## place)" (AN-22's `put`); that `put` changes `this.pools` is worked out from its body, and its first
## line says nothing about it (R5, verdict 86). The pool made for the map's `owns dyn AnyPool` values
## says `owns` (verdict 85).

interface AnyPool:
    fn count() -> int

[T]
struct Pool implements AnyPool:
    dense: Array[T]
    sparse: Array[int]

    fn count() -> int:
        return this.dense.len()

struct Registry:
    pools: Map[u128, owns dyn AnyPool]

    [T]
    fn put(k: u128, e: int, owns c: T):
        if this.pools[k] is Pool[T] as ref mut pool:
            pool.dense[pool.sparse[e]] = c

fn main():
    reg = Registry(pools={1u128: owns Pool[String](dense=[String("")], sparse=[0])})
    reg.put(1u128, 0, String("ok"))
    if reg.pools[1u128] is Pool[String] as p:
        println(p.dense[0])
