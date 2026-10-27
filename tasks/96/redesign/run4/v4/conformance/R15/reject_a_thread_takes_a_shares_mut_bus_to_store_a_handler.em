#$ test: compile-fail
#$ rules: R15, R9
## R15: "`with shares`, `with shares mut` and `with weakref` only where the function stays on this
## thread"; a `shares mut` owner captured by an unscoped thread is E7015 (completeness G26, the duty's
## case as written).

import std.thread

class Bus:
    handlers: Array[fn()] = []

    fn on_event(owns f: fn()) changes handlers:
        this.handlers.push(f)

fn ping():
    println("ping")

fn main() changes Bus:
    bus = shares mut Bus()
    t = thread.spawn(fn() with shares mut bus changes Bus: bus.on_event(ping))    #$ error[E7015]: `with shares mut bus` makes an owner of `bus` that stays on one thread, and this function runs on a new one; share the bus between threads: write `from std.sync import Lock` at the top; in `Bus`, `lock: Lock = Lock()` and `handlers: Array[thread_shared fn()] guarded by lock = []` instead of line 10, `fn on_event(owns f: thread_shared fn()) changes handlers:` instead of line 12 and `with this.lock:` above line 13, with line 13 inside it; `bus = thread_shared Bus()` instead of line 19, and `t = thread.spawn(fn() with thread_shared bus changes Bus: bus.on_event(ping))` instead of line 20
    t.join()
