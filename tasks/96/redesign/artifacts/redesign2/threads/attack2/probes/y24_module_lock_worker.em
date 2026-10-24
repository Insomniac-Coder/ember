# Proposed syntax, readability attack on A19 rule 5 (answers report 6.3: "a function reachable from
# thread.spawn ... that touches a module variable is a compile-time error"). Python's most common threading
# shape: a module lock and a module list; workers append under the lock.
import std.thread
from std.sync import Lock

RESULTS_LOCK = Lock()
RESULTS: Array[int] = []

fn worker(x: int):
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main():
    hs = [thread.spawn(fn(): worker(i)) for i in 0..4]
    for h in hs:
        h.join()
    with RESULTS_LOCK:
        println(len(RESULTS))
