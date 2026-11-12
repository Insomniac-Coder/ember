#$ test: run-pass
#$ rules: THR-16, THR-10
#$ profiles: debug
#$ stdout: main ends
#$ stderr: no leak report: 1 other thread is still running
# `[THR-16]` — when `main` returns the process ends, and a thread still running (this one sleeps
# far longer than `main` runs) stops without running destructors: its `Noisy` never says it was
# dropped. The debug leak report runs only when no other thread is running; otherwise it says how
# many were.

import std.thread
from std.time import Duration, sleep

struct Noisy:
    pub id: int

    fn drop():
        println("dropped", this.id)

fn wait_forever():
    _held = Noisy(id=1)
    sleep(Duration.from_secs(3600))
    println("never printed")

fn main():
    _waiter = thread.spawn(owns owned fn() => wait_forever())
    println("main ends")
