# Clause THR-3.1: "A guard belongs to the thread that took the lock: sending it to another thread is
# `E7008`"; "A `Mutex`, `RwLock`, `Atomic` or channel end is changed through any name, a plain one
# included, by its own operations, and an argument given to one takes no word at the call (`ref mut`
# before it is `E3028`"
#$ test: compile-fail
#$ rules: THR-3, FN-2a, THR-11
# The spec's two messages, with this file's line numbers.

import std.thread
from std.sync import Mutex

fn top(scores: Mutex[Array[int]]) -> int:
    with g = scores.lock():
        return g[0]

fn main():
    m = Mutex([0])
    with g = m.lock():
        with scope = thread.scope():
            scope.spawn(owns fn(): g.push(owns 1))    #$ error[E7008]: `g` is the guard of `m`'s lock, which belongs to the thread that took it; take the lock inside the task: write `fn add_one(m: Mutex[Array[int]]):` with the body `with g = m.lock():` and, inside it, `g.push(owns 1)`, above line 16, and `scope.spawn(owns fn(): add_one(m))` in place of line 20
    scores = Mutex([3])
    println(top(ref mut scores))    #$ error[E3028]: `top` takes the lock of `scores` itself; a `Mutex` takes no word at the call: `top(scores)`
