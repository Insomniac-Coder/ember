#$ test: run-pass
#$ rules: THR-2, THR-1, THR-10
#$ stdout: 2
# `[THR-2]` — a handle of a `@sync` class is `Send` and `Sync`: a thread's function may capture it.

import std.thread
from std.sync import Atomic

class Counter:
    hits: Atomic[int] = Atomic(0)

fn main():
    counter = thread_shared Counter()
    mine = thread_shared counter
    worker = thread.spawn(owns owned fn() => mine.hits.fetch_add(2))
    worker.join()
    println(counter.hits.load())
