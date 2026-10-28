#$ test: compile-fail
#$ rules: R15, R12
## R12 and R15 (verdict 61): "A function of such a type [`thread_shared fn()`] may hold `thread_shared`
## values or copies, never a `shares` or `shares mut` value ... (E7015)"; section 3's E7015 message, in
## this program's lines (completeness G26: the one-thread owner hidden in a function value's state).

import std.thread
from std.sync import Lock

class Counter:
    n: int = 0

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
    count = shares mut Counter()
    bus.on_event(fn() with shares mut count changes Counter: count.n += 1)    #$ error[E7015]: `on_event` keeps functions any thread may run (`thread_shared fn() changes Counter`), and this one holds `count`, a `shares mut` owner, which stays on one thread; share the counter between threads: write `from std.sync import Atomic, Lock` instead of line 8, `n: Atomic[int] = Atomic(0)` instead of line 11, `count = thread_shared Counter()` instead of line 28, `bus.on_event(fn() with thread_shared count changes Counter: count.n.fetch_add(1))` instead of line 29, and `println(count.n.load())` instead of line 32
    t = thread.spawn(fn() with thread_shared bus changes Bus, Counter: bus.fire())
    t.join()
    println(count.n)
