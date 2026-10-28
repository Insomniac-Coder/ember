#$ test: compile-fail
#$ rules: BRW-15
## design rules: R14, R5
## R14 with R5 (verdict 86): `put` changes `this.pools` through the `ref mut` name its kind test gives,
## so the compiler works out that `put` changes its object, and "A method that may change its object is
## not called through a name that only reads it (E3023)" (AN-22's `put`; verdict 86 moved the check
## from a word on `put`'s first line, E2105, to this call).

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

fn fill(reg: Registry):
    reg.put(1u128, 0, String("ok"))    #$ error[E3023]: `put` changes the registry (line 25, through `pool`, a `ref mut` name of `this.pools[k]`), and `reg` only reads it; write `fn fill(ref mut reg: Registry):` in place of line 27, and call it `fill(ref mut reg)` in place of line 32

fn main():
    reg = Registry(pools={1u128: owns Pool[String](dense=[String("")], sparse=[0])})
    fill(reg)
