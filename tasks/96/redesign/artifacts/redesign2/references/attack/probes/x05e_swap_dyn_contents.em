# Attack on the save of the pool table: if its iteration hands out the CONTENTS of each box as
# `ref mut dyn AnyPool`, can user code swap two contents of different types? (It must not.)
import std.mem

interface AnyPool:
    fn count(self) -> int

struct Pool[T]:
    dense: Array[T]

extend[T] Pool[T] implements AnyPool:
    fn count(self) -> int:
        return self.dense.len()

struct Big:
    a: Array[int]

fn swap_contents(a: ref mut dyn AnyPool, b: ref mut dyn AnyPool):
    mem.swap(a, b)

fn main():
    x: Box[dyn AnyPool] = Box(Pool[int](dense=[1]))
    y: Box[dyn AnyPool] = Box(Pool[Big](dense=[]))
    swap_contents(ref mut x.get(), ref mut y.get())
    println(x.count())
