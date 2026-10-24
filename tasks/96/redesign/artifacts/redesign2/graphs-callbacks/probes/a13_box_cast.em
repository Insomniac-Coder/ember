# A13: today's registry with Box[dyn] pools and a cast (phase 1's a13_dyn.em, trimmed).
from std.collections import Map

interface AnyPool:
    fn count(self) -> int

struct Pool[T]:
    dense: Array[T]

extend[T] Pool[T] implements AnyPool:
    fn count(self) -> int:
        return self.dense.len()

struct Reg:
    pools: Map[u128, Box[dyn AnyPool]]

    fn get_mut[T](mut self, i: int) -> ref mut T:
        pool = cast self.pools[7u128] as Pool[T]
        return ref mut pool.dense[i]

fn main():
    reg = Reg(pools=Map[u128, Box[dyn AnyPool]]())
    _none = reg.pools.insert(7u128, Box(Pool[int](dense=[10, 20])))
    p = reg.get_mut[int](1)
    p += 1
