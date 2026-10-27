#$ test: compile-fail
#$ rules: THR-1, THR-2, THR-10, CLO-16, FFI-22
#$ help: handlers: Array[thread_shared fn()] guarded by lock = []
## design rules: R12, R15
## Step 3's hole 6a, its original form (`safety/recheck.md` section 5), R12 (rules.md lines 698-702): "A
## class whose objects are made `thread_shared` holds functions only in such types: `bus = thread_shared
## Bus()`, for a `class Bus:` with `handlers: Array[fn()] = []`, is refused at that line (E7015)": a
## plain `fn()` may hold a one-thread owner, and another thread (here the one `fire` runs on) would run
## it. rules.md section 3's line 1103 second message, in this program's lines; it also names `on_event`'s
## line, since a plain `fn()` does not go into a list of `thread_shared fn()` (R18: the fix compiles).
## The nearest allowed program is accept_a_bus_with_thread_shared_fn_handlers_made_thread_shared.

import std.thread
from std.sync import Lock

class Bus:
    lock: Lock = Lock()
    handlers: Array[fn()] guarded by lock = []

    fn on_event(owns f: fn()):
        with this.lock:
            this.handlers.push(f)

    fn fire():
        with this.lock:
            for ref mut h in this.handlers:
                h()

fn main():
    bus = thread_shared Bus()    #$ error[E7015]: `Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any thread; write `handlers: Array[thread_shared fn()] guarded by lock = []` in `Bus` in place of line 17, and `fn on_event(owns f: thread_shared fn()):` in place of line 19
    bus.on_event(fn(): println("event"))
    t = thread.spawn(fn() thread_shared bus: bus.fire())
    t.join()
