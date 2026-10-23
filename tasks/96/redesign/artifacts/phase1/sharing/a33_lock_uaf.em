# A33 follow-up: does the reference returned from inside the lock outlive the guard?
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        return ref g[0]

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    r = top(m)
    with g = m.lock():
        g.clear()
    println(r)
