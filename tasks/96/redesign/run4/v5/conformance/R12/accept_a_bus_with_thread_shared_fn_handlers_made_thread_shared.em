#$ test: run-pass
#$ rules: THR-1, THR-2, THR-10, CLO-16, FFI-22
#$ stdout: event
## design rules: R12, R15
## Step 3's hole 6a, its original form, R12 (rules.md lines 692-702): a `thread_shared` object holds its
## functions in a `thread_shared fn()` type, so every function stored there holds only `thread_shared`
## values or copies, and another thread may run them (here `fire`, on the thread `spawn` starts); the list
## is `guarded by` the bus's lock (R15). The lines the E7015 message of
## reject_a_bus_with_plain_fn_handlers_made_thread_shared gives, and nothing else changed.

import std.thread
from std.sync import Lock

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
    bus.on_event(fn(): println("event"))
    t = thread.spawn(fn() thread_shared bus: bus.fire())
    t.join()
