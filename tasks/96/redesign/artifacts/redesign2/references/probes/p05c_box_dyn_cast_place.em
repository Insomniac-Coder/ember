# A5 / ODR-286 with the owner's verdict (pools in Box[dyn AnyPool]): downcast of a place.
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
