#$ test: run-pass
#$ rules: R15, R12, R4, R5
#$ stdout: 3
## R15: the lines E7008 gives: the task takes the lock itself, after the parent's `with g` block ends; a
## `Mutex` takes no word at the call (R4); what a function changes in one is named by its type
## (`changes Mutex[Array[int]]`, R12), and a change a callee names by a type counts for its caller,
## its own local included (R5), so the task and `main` name it.

import std.thread
from std.sync import Mutex

fn add_one(m: Mutex[Array[int]]) changes Mutex[Array[int]]:
    with g = m.lock():
        g.push(1)

fn main() changes Mutex[Array[int]]:
    m = Mutex([0])
    with scope = thread.scope():
        with g = m.lock():
            g.push(2)
            scope.spawn(fn() changes Mutex[Array[int]]: add_one(m))
    with g = m.lock():
        println(g.len())
