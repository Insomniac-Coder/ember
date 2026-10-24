# Diagnostic check: swapping two elements of one Map is refused (two writable borrows of `pools`), but the
# message talks about methods borrowing the fields of `self` ([BRW-10]), and no method of self is involved.
from std.collections import Map
import std.mem

fn main():
    pools = Map[int, String]()
    _a = pools.insert(1, "a")
    _b = pools.insert(2, "b")
    mem.swap(pools[1], pools[2])
    println(pools[1])
