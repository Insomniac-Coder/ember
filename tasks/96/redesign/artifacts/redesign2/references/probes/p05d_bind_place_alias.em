# A5: binding a place that cannot be moved from (a map value behind mut self) without `ref mut`.
from std.collections import Map

struct Pool[T]:
    dense: Array[T]

struct Reg:
    pools: Map[u128, Box[Pool[int]]]

    fn get_mut(mut self, i: int) -> ref mut int:
        pool = self.pools[7u128]
        return ref mut pool.dense[i]

fn main():
    reg = Reg(pools=Map[u128, Box[Pool[int]]]())
    _none = reg.pools.insert(7u128, Box(Pool[int](dense=[10, 20])))
    p = reg.get_mut(1)
    p += 1
