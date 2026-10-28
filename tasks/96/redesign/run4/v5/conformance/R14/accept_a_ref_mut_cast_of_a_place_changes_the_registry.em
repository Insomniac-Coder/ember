#$ test: run-pass
#$ rules: TYP-44, BRW-12, LT-1
#$ stdout: 11
## design rules: R14, R2, R6
## R14: "A cast of a place is a place" (AN-22's `get_mut`): `ref mut cast` names the registry's own
## pool, so the element returned `ref mut` points into `reg`, and `p += 1` writes it (R2, R6). The pool
## made for the map's `owns dyn AnyPool` values says `owns` (verdict 85).

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
    fn get_mut(k: u128, e: int) -> ref mut T:
        pool = ref mut cast this.pools[k] as Pool[T]
        return ref mut pool.dense[pool.sparse[e]]

fn main():
    reg = Registry(pools={7u128: owns Pool[int](dense=[10], sparse=[0])})
    p = ref mut reg.get_mut[int](7u128, 0)
    p += 1
    if reg.pools[7u128] is Pool[int] as q:
        println(q.dense[0])
