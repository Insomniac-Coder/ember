#$ test: compile-fail
#$ rules: R14, R5
## R14 with R5: a `ref mut` kind-test name may change the place only "where the function may change
## the place"; `this` in a method whose line says no `changes` only reads, and a change through it is
## E2105 (AN-22's `put` without `changes pools`).

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
    fn put(k: u128, e: int, owns c: T):    #$ error[E2105]: `put` changes `this.pools` through `pool`; write its first line as `[T]` above `fn put(k: u128, e: int, owns c: T) changes pools:`
        if this.pools[k] is Pool[T] as ref mut pool:
            pool.dense[pool.sparse[e]] = c

fn main():
    reg = Registry(pools={1u128: Pool[String](dense=[String("")], sparse=[0])})
    reg.put(1u128, 0, String("ok"))
