#$ test: compile-fail
#$ rules: R14, R5
## R14: "plain `c` is a read-only view, and since a value tested in its place stays a place,
## `if this.pools[k] is Pool[T] as ref mut pool:` names a view of the registry's pool that may change
## it" (AN-22's `put`, with the name written plain).

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
    fn put(k: u128, e: int, owns c: T) changes pools:
        if this.pools[k] is Pool[T] as pool:
            pool.dense[pool.sparse[e]] = c    #$ error[E3023]: `pool` reads the pool in `this.pools` and cannot change it; to store the component through it, write `if this.pools[k] is Pool[T] as ref mut pool:`

fn main():
    reg = Registry(pools={1u128: Pool[String](dense=[String("")], sparse=[0])})
    reg.put(1u128, 0, String("ok"))
