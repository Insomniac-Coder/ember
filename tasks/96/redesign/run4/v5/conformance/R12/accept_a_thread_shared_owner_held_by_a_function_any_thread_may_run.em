#$ test: run-pass
#$ rules: R12, R15
#$ stdout: 1
## Verdict 61, R12 (rules.md lines 692-697): a function any thread may run holds a `thread_shared` owner of
## the counter, whose number is an `Atomic`, changed through any name by its own operation (R5). The lines
## the E7015 message of reject_a_shares_mut_owner_held_by_a_function_any_thread_may_run gives, and nothing
## else changed.

from std.sync import Atomic

class Counter:
    n: Atomic[int] = Atomic(0)

class Bus:
    handlers: Array[thread_shared fn()] = []

    fn on_event(owns f: thread_shared fn()):
        this.handlers.push(f)

    fn fire():
        for ref mut h in this.handlers:
            h()

fn main():
    bus = Bus()
    count = thread_shared Counter()
    bus.on_event(fn() thread_shared count: count.n.fetch_add(1))
    bus.fire()
    println(len(bus.handlers))
