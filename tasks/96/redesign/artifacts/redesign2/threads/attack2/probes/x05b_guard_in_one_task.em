# Lock held by the parent; one scoped task writes through the parent's guard (Rust accepts this
# through `&mut *g`; it is race-free, the parent does not touch g meanwhile).
import std.thread
from std.sync import Mutex

fn main():
    m: Mutex[Array[int]] = Mutex([])
    with g = m.lock():
        with scope = thread.scope():
            scope.spawn(fn(): g.push(1))
    with g = m.lock():
        println(g.len())
