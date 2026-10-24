# Attack on A5's pool table keyed by the type: its read does an unchecked downcast, so it is safe only
# if no code can ever put a Pool[Vel] in the Pool[Pos] slot. A table built on a Map of boxes that hands
# out its slots lets user code replace the Pool[Pos] box with a Pool[Big] box; so the table must never hand out a slot.
from std.collections import Map
import std.mem

interface AnyPool:
    fn count(self) -> int

struct Pool[T]:
    dense: Array[T]

extend[T] Pool[T] implements AnyPool:
    fn count(self) -> int:
        return self.dense.len()

struct Pos:
    x: f32
    y: f32

struct Big:
    a: Array[int]

fn main():
    pools = Map[int, Box[dyn AnyPool]]()
    _a = pools.insert(1, Box(Pool[Pos](dense=[])))
    _b = pools.insert(2, Box(Pool[Big](dense=[])))
    _old = mem.replace(pools[1], Box(Pool[Big](dense=[])))
    println(pools[1].count())
