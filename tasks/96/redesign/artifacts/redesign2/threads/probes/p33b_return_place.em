# A33: the Python spelling `return g[0]` with a value result type (no clone written).
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return g[0]

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    println(top(m))
