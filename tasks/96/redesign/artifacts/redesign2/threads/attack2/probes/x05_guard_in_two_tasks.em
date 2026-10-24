# Lock held by the parent; two scoped tasks write through the parent's guard.
# Today: refused by "a guard is not Send" ([THR-3]) and two mutable borrows. The proposal drops Send.
import std.thread
from std.sync import Mutex

fn main():
    m: Mutex[Array[int]] = Mutex([])
    with g = m.lock():
        with scope = thread.scope():
            scope.spawn(fn(): g.push(1))
            scope.spawn(fn(): g.push(2))
    with g = m.lock():
        println(g.len())
