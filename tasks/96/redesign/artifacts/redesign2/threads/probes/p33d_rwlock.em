# A33: the same hole through RwLock's read guard.
from std.sync import RwLock

fn top(scores: RwLock[Array[String]]) -> ref String:
    with g = scores.read():
        return ref g[0]

fn clear(scores: RwLock[Array[String]]):
    with g = scores.write():
        g.clear()

fn main():
    m: RwLock[Array[String]] = RwLock(["ann".to_string()])
    r = top(m)
    clear(m)
    println(r)
