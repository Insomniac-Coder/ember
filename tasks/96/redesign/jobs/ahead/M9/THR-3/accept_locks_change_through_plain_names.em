# Clause THR-3.1: "A `Mutex`, `RwLock`, `Atomic` or channel end is changed through any name, a plain
# one included, by its own operations"; the E7008 message's fix
#$ test: run-pass
#$ rules: THR-3, THR-11, THR-14
#$ stdout: 2
#$ stdout: 3
#$ stdout: 7
# A task takes the lock itself (the message's line), through a plain parameter; an `RwLock` written
# and read; an `Atomic` through a plain parameter.

import std.thread
from std.sync import Mutex, RwLock, Atomic

fn add_one(m: Mutex[Array[int]]):
    with g = m.lock():
        g.push(owns 1)

fn bump(a: Atomic[int]):
    a.fetch_add(4)

fn main():
    m = Mutex([0])
    with scope = thread.scope():
        scope.spawn(owns fn(): add_one(m))
    with g = m.lock():
        println(g.len())
    rw = RwLock(1)
    with w = rw.write():
        w += 2
    with r = rw.read():
        println(r)
    a = Atomic(3)
    bump(a)
    println(a.load())
