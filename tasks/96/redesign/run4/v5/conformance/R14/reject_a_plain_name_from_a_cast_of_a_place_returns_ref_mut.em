#$ test: compile-fail
#$ rules: R14, R2
## R14: "A cast of a place is a place: the name bound from it is read-only unless written `ref mut`
## (`pool = ref mut cast this.pools[k] as Pool[T]`), and `ref mut` taken through a plain one
## (`return ref mut pool.dense[pool.sparse[e]]`) is refused (E3027)" (AN-22's `get_mut`, with `ref mut`
## left out; the code is the lead's, worklist section 3, U6). The pool made for the map's
## `owns dyn AnyPool` values says `owns` (verdict 85).

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
        pool = cast this.pools[k] as Pool[T]
        return ref mut pool.dense[pool.sparse[e]]    #$ error[E3027]: `pool` only reads the pool in `this.pools`, so `ref mut pool.dense[...]` cannot be taken through it; name it so it may change: write `pool = ref mut cast this.pools[k] as Pool[T]` in place of line 25

fn main():
    reg = Registry(pools={7u128: owns Pool[int](dense=[10], sparse=[0])})
    p = ref mut reg.get_mut[int](7u128, 0)
    p += 1
