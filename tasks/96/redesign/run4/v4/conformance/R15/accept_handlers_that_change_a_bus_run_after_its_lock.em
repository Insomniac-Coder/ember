#$ test: run-pass
#$ rules: R15, R12
#$ stdout: 1
## R15: the lines E7018 gives: the handlers are taken out under the lock and run after it, so a handler
## that changes a `Bus`, this one included, takes the lock itself; they are put back under the lock.

import std.thread
import std.mem
from std.sync import Lock

class Bus:
    lock: Lock = Lock()
    handlers: Array[thread_shared fn() changes Bus] guarded by lock = []

    fn on_event(owns f: thread_shared fn() changes Bus) changes handlers:
        with this.lock:
            this.handlers.push(f)

    fn fire() changes handlers, Bus:
        with this.lock:
            todo = mem.take(ref mut this.handlers)
        for ref mut h in todo:
            h()
        with this.lock:
            this.handlers.extend(todo)

fn main() changes Bus:
    other = thread_shared Bus()
    fn poke() with thread_shared other changes Bus:
        other.on_event(fn(): println("ping"))
    bus = thread_shared Bus(handlers=[poke])
    t = thread.spawn(fn() with thread_shared bus changes Bus: bus.fire())
    t.join()
    with other.lock:
        println(other.handlers.len())
