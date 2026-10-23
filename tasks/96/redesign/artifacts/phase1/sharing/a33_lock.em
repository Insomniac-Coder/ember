# A33: a shared score table behind a lock; hand out the top name.
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        return ref g[0]

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    println(top(m))
