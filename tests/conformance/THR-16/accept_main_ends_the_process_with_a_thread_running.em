#$ test: run-pass
#$ rules: THR-16, THR-10
#$ profiles: debug
#$ stdout: main ends
#$ stderr: no leak report: 1 other thread is still running
# `[THR-16]` — when `main` returns the process ends, and a thread still running (this one waits
# for a lock `main` never lets go) stops without running destructors. The debug leak report runs
# only when no other thread is running; otherwise it says how many were.

import std.thread
from std.sync import Mutex, SyncShared

fn wait_forever(gate: SyncShared[Mutex[int]]):
    with g = gate.get().lock():
        println("never printed", g)

fn main():
    gate = SyncShared(Mutex(0))
    held = gate.get().lock()
    theirs = gate
    _waiter = thread.spawn(owned fn() => wait_forever(theirs))
    mem.forget(held)
    println("main ends")
