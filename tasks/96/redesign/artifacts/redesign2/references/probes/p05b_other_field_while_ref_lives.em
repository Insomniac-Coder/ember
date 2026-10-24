# A5: the returned reference points into self.pools only; using another field while it lives.
from std.collections import Map

struct Pool[T]:
    dense: Array[T]

struct Reg:
    pools: Map[u128, Box[Pool[int]]]
    next_id: int

    fn get_mut(mut self, i: int) -> ref mut int:
        pool = ref mut self.pools[7u128]
        return ref mut pool.dense[i]

fn main():
    reg = Reg(pools=Map[u128, Box[Pool[int]]](), next_id=0)
    _none = reg.pools.insert(7u128, Box(Pool[int](dense=[10, 20])))
    p = reg.get_mut(1)
    reg.next_id += 1
    p += 1
    println(reg.pools[7u128].dense[1])
