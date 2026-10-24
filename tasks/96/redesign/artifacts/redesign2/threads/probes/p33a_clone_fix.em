# A33: the fix's spelling: a copy leaves the lock.
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return g[0].clone()

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    println(top(m))
