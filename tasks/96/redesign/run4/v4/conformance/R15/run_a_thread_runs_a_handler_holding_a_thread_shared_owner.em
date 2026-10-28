#$ test: run-pass
#$ rules: R15, R12, R9
#$ stdout: 1
## R12 and R15 (verdict 61): the lines E7015 gives: a `thread_shared fn()` holds a `thread_shared`
## owner, and the counter it changes is an `Atomic`, changed by its own operation; another thread runs
## it under the bus's lock (completeness G26's nearest allowed; ThreadSanitizer: no report).

import std.thread
from std.sync import Atomic, Lock

class Counter:
    n: Atomic[int] = Atomic(0)

class Bus:
    lock: Lock = Lock()
    handlers: Array[thread_shared fn() changes Counter] guarded by lock = []

    fn on_event(owns f: thread_shared fn() changes Counter) changes handlers:
        with this.lock:
            this.handlers.push(f)

    fn fire() changes handlers, Counter:
        with this.lock:
            for ref mut h in this.handlers:
                h()

fn main() changes Bus, Counter:
    bus = thread_shared Bus()
    count = thread_shared Counter()
    bus.on_event(fn() with thread_shared count changes Counter: count.n.fetch_add(1))
    t = thread.spawn(fn() with thread_shared bus changes Bus, Counter: bus.fire())
    t.join()
    println(count.n.load())
