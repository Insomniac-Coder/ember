# A33 follow-up, one thread: the reference from inside the lock, then a call that clears the list.
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
    clear(m)
    println(r)
