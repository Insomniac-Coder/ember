#$ test: run-pass
#$ rules: R14, R5, R4
#$ stdout: ok
## R14 with R5 and R4: the lines E3023 gives: `fill` takes the registry `ref mut`, the call says so, and
## `put`, which changes `this.pools` through its kind test's `ref mut` name, is called through it.

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

fn fill(ref mut reg: Registry):
    reg.put(1u128, 0, String("ok"))

fn main():
    reg = Registry(pools={1u128: owns Pool[String](dense=[String("")], sparse=[0])})
    fill(ref mut reg)
    if reg.pools[1u128] is Pool[String] as p:
        println(p.dense[0])
