#$ test: run-pass
#$ rules: R15, R12
#$ stdout: 1
## R15 and R12 (verdict 61): the lines E7015 gives: the bus is `thread_shared`, its handlers field is
## guarded by its lock and holds `thread_shared fn()` functions (the fields of a `thread_shared` object
## that hold functions use such types), and the thread holds one more owner of it.

import std.thread
from std.sync import Lock

class Bus:
    lock: Lock = Lock()
    handlers: Array[thread_shared fn()] guarded by lock = []

    fn on_event(owns f: thread_shared fn()):
        with this.lock:
            this.handlers.push(f)

fn ping():
    println("ping")

fn main():
    bus = thread_shared Bus()
    t = thread.spawn(fn() thread_shared bus: bus.on_event(ping))
    t.join()
    with bus.lock:
        println(bus.handlers.len())
