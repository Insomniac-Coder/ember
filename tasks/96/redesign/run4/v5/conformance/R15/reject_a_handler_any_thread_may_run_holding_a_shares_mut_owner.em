#$ test: compile-fail
#$ rules: R15, R12
## R12 and R15 (verdict 61): "A function of such a type [`thread_shared fn()`] may hold `thread_shared`
## values or copies, never a `shares` or `shares mut` value ... (E7015)"; section 3's E7015 message, in
## this program's lines (completeness G26: the one-thread owner hidden in a function value's state). The
## nearest allowed program is `run_a_thread_runs_a_handler_holding_a_thread_shared_owner.em`.

import std.thread
from std.sync import Lock

class Counter:
    n: int = 0

class Bus:
    lock: Lock = Lock()
    handlers: Array[thread_shared fn()] guarded by lock = []

    fn on_event(owns f: thread_shared fn()):
        with this.lock:
            this.handlers.push(f)

    fn fire():
        with this.lock:
            for ref mut h in this.handlers:
                h()

fn main():
    bus = thread_shared Bus()
    count = shares mut Counter()
    bus.on_event(fn() shares mut count: count.n += 1)    #$ error[E7015]: `on_event` keeps functions any thread may run (`thread_shared fn()`), and this one holds `count`, a `shares mut` owner, which stays on one thread; share the counter between threads: write `from std.sync import Atomic, Lock` instead of line 9, `n: Atomic[int] = Atomic(0)` instead of line 12, `count = thread_shared Counter()` instead of line 29, `bus.on_event(fn() thread_shared count: count.n.fetch_add(1))` instead of line 30, and `println(count.n.load())` instead of line 33
    t = thread.spawn(fn() thread_shared bus: bus.fire())
    t.join()
    println(count.n)
