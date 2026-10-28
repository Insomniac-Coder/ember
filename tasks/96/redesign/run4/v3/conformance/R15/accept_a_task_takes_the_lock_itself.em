#$ test: run-pass
#$ rules: R15, R12, R4
#$ stdout: 2
## R15: the line E7008 gives: the task takes the lock itself; a `Mutex` takes no word at the call (R4)
## and what a function changes in one is named (`changes Mutex[Array[int]]`, R12).

import std.thread
from std.sync import Mutex

fn add_one(m: Mutex[Array[int]]) changes Mutex[Array[int]]:
    with g = m.lock():
        g.push(1)

fn main():
    m = Mutex([0])
    with scope = thread.scope():
        scope.spawn(fn() changes m: add_one(m))
    with g = m.lock():
        println(g.len())
