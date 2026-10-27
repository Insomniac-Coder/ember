#$ test: compile-fail
#$ rules: R15, R12
## R15 (verdicts 43 and 86): a call through a function value that may run a function taking a lock this
## thread holds is E7018: `h()` may run `poke`, which takes a `Bus`'s lock, inside `with this.lock:`;
## inside the walk of `this.handlers` too, E7018 and not E3020 (R18, verdict 73) (completeness G26; the
## lines of the way out are written here). The handler is put in the bus where the bus is made. The
## nearest allowed program is `accept_handlers_that_change_a_bus_run_after_its_lock.em`.

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
                h()    #$ error[E7018]: this loop walks `this.handlers` (line 22) under `this.lock`, and `h()` (line 23) may run `poke`, stored on line 29, which takes a `Bus`'s lock (through `on_event`, line 17), so it would wait for the lock this thread holds; take the handlers out under the lock and run them after it: write `import std.mem` below line 9, and instead of lines 21 to 23 write `with this.lock:` with `todo = mem.take(ref mut this.handlers)` inside it, then `for ref mut h in todo:` with `h()` inside it, then `with this.lock:` with `this.handlers.extend(todo)` inside it

fn main():
    other = thread_shared Bus()
    fn poke() thread_shared other:
        other.on_event(fn(): println("ping"))
    bus = thread_shared Bus(handlers=[poke])
    t = thread.spawn(fn() thread_shared bus: bus.fire())
    t.join()
