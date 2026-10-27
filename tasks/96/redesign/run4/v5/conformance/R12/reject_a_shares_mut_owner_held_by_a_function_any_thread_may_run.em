#$ test: compile-fail
#$ rules: R12, R15
#$ help: fn() thread_shared count: count.n.fetch_add(1)
## Verdict 61, R12 (rules.md lines 692-697): "A function of such a type may hold `thread_shared` values or
## copies, never a `shares` or `shares mut` value or a `weakref` link of one (E7015): `bus.on_event(fn()
## shares mut count: count.n += 1)` is refused, since another thread running it would change `count`'s
## one-thread count and value" (as Rust's `Send`). rules.md section 3's line 1103 first message, in this
## program's lines (completeness G26's case, at the store). The nearest allowed program is
## accept_a_thread_shared_owner_held_by_a_function_any_thread_may_run.

class Counter:
    n: int = 0

class Bus:
    handlers: Array[thread_shared fn()] = []

    fn on_event(owns f: thread_shared fn()):
        this.handlers.push(f)

    fn fire():
        for ref mut h in this.handlers:
            h()

fn main():
    bus = Bus()
    count = shares mut Counter()
    bus.on_event(fn() shares mut count: count.n += 1)    #$ error[E7015]: `on_event` keeps functions any thread may run (`thread_shared fn()`), and this one holds `count`, a `shares mut` owner, which stays on one thread; share the counter between threads: write `count = thread_shared Counter()` in place of line 26, give `Counter` the field `n: Atomic[int] = Atomic(0)` in place of line 12 (with `from std.sync import Atomic` at the top), and pass `fn() thread_shared count: count.n.fetch_add(1)`
    bus.fire()
    println(len(bus.handlers))
