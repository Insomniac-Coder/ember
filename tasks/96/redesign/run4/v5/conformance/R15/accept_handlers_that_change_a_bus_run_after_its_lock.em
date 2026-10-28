#$ test: run-pass
#$ rules: THR-1, CLO-1
#$ stdout: 1
## design rules: R15, R12
## R15: the lines E7018 gives: the handlers are taken out under the lock and run after it, so a handler
## that changes a `Bus`, this one included, takes the lock itself; they are put back under the lock.

import std.thread
import std.mem
from std.sync import Lock

class Bus:
    lock: Lock = Lock()
    handlers: Array[thread_shared fn()] guarded by lock = []

    fn on_event(owns f: thread_shared fn()):
        with this.lock:
            this.handlers.push(f)

    fn fire():
        with this.lock:
            todo = mem.take(ref mut this.handlers)
        for ref mut h in todo:
            h()
        with this.lock:
            this.handlers.extend(todo)

fn main():
    other = thread_shared Bus()
    fn poke() thread_shared other:
        other.on_event(fn(): println("ping"))
    bus = thread_shared Bus(handlers=[poke])
    t = thread.spawn(fn() thread_shared bus: bus.fire())
    t.join()
    with other.lock:
        println(other.handlers.len())
