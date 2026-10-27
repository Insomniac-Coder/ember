#$ test: compile-fail
#$ rules: R15, R12
## R15 (verdict 43): a call through a function value whose type changes a type whose lock this thread
## holds is E7018: `h()` may change any `Bus`, this one included, inside `with this.lock:`; inside the
## walk of `this.handlers` too, E7018 and not E3020 (R18, verdict 73) (completeness G26; the lines of
## the way out are written here). The handler is put in the bus where the bus is made, so no call has
## the bus and a `changes Bus` function as two arguments (R4).

import std.thread
from std.sync import Lock

class Bus:
    lock: Lock = Lock()
    handlers: Array[thread_shared fn() changes Bus] guarded by lock = []

    fn on_event(owns f: thread_shared fn() changes Bus) changes handlers:
        with this.lock:
            this.handlers.push(f)

    fn fire() changes handlers, Bus:
        with this.lock:
            for ref mut h in this.handlers:
                h()    #$ error[E7018]: this loop walks `this.handlers` (line 22) under `this.lock`, and `h()` (line 23) may change any `Bus`, this one included (the handlers' type says `changes Bus`), so it would wait for the lock this thread holds; take the handlers out under the lock and run them after it: write `import std.mem` at the top, and instead of lines 21 to 23 write `with this.lock:` with `todo = mem.take(ref mut this.handlers)` inside it, then `for ref mut h in todo:` with `h()` inside it, then `with this.lock:` with `this.handlers.extend(todo)` inside it

fn main() changes Bus:
    other = thread_shared Bus()
    fn poke() with thread_shared other changes Bus:
        other.on_event(fn(): println("ping"))
    bus = thread_shared Bus(handlers=[poke])
    t = thread.spawn(fn() with thread_shared bus changes Bus: bus.fire())
    t.join()
