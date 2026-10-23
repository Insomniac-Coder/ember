# A13: an ECS registry of type-erased pools, found by type; and an interface with a generic method.
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

interface Visitor:
    fn visit(mut self, n: int)

interface Inspect:
    fn inspect[V: Visitor](self, mut v: V)

fn all(xs: Array[Box[dyn Inspect]]) -> int:
    return xs.len()

fn main():
    reg = Reg(pools=Map[u128, Box[dyn AnyPool]]())
    _none = reg.pools.insert(7u128, Box(Pool[int](dense=[10, 20])))
    p = reg.get_mut[int](1)
    p += 1
