# A33 proposed (b): an accessor that keeps the lock while the caller uses the name.
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        yield g[0]

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    with name = top(m):
        println(name)
