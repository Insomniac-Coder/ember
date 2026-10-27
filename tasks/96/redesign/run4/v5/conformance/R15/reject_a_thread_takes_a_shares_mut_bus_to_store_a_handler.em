#$ test: compile-fail
#$ rules: THR-1, THR-2, THR-10, CLO-16, FFI-22
## design rules: R15, R9, R12
## R15 (verdict 94): "`thread.spawn` takes a function any thread may run ..., so a function holding a
## one-thread value is refused by R12's rule (E7015)": a `shares mut` owner captured by an unscoped
## thread (completeness G26, the duty's case as written). The nearest allowed program is
## `accept_a_thread_stores_a_handler_in_a_bus_threads_share.em`.

import std.thread

class Bus:
    handlers: Array[fn()] = []

    fn on_event(owns f: fn()):
        this.handlers.push(f)

fn ping():
    println("ping")

fn main():
    bus = shares mut Bus()
    t = thread.spawn(fn() shares mut bus: bus.on_event(ping))    #$ error[E7015]: `shares mut bus` makes an owner of `bus` that stays on one thread, and `thread.spawn` runs this function on a new one; share the bus between threads: write `from std.sync import Lock` below line 8; in `Bus`, `lock: Lock = Lock()` and `handlers: Array[thread_shared fn()] guarded by lock = []` instead of line 11, `fn on_event(owns f: thread_shared fn()):` instead of line 13 and `with this.lock:` above line 14, with line 14 inside it; `bus = thread_shared Bus()` instead of line 20, and `t = thread.spawn(fn() thread_shared bus: bus.on_event(ping))` instead of line 21
    t.join()
