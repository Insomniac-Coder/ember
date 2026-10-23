# A33 follow-up: a reference returned from inside the lock, read while a scoped task locks and clears.
import std.thread
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        return ref g[0]

fn clear(scores: Mutex[Array[String]]):
    with g = scores.lock():
        g.clear()

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    r = top(m)
    with scope = thread.scope():
        scope.spawn(fn() => clear(m))
        println(r)
