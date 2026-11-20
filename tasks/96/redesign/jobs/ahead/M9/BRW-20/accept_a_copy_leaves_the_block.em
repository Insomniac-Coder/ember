# Clause BRW-20.1: "return a copy: `return copy of g[0]`, with the result type `String`"
#$ test: run-pass
#$ rules: BRW-20, DIA-25
#$ stdout: a
#$ stdout: b
# The messages' fix.

from std.sync import Mutex, RwLock

fn first_locked(m: Mutex[Array[String]]) -> String:
    with g = m.lock():
        return copy of g[0]

fn first_read(rw: RwLock[Array[String]]) -> String:
    with g = rw.read():
        return copy of g[0]

fn main():
    m = Mutex([String("a")])
    println(first_locked(m))
    rw = RwLock([String("b")])
    println(first_read(rw))
