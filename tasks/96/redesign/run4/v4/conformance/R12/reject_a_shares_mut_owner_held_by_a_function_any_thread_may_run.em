#$ test: compile-fail
#$ rules: R12, R15
#$ help: fn() with thread_shared count changes Counter: count.n.fetch_add(1)
## Verdict 61, R12 (rules.md lines 651-658): a list whose functions any thread may run says
## `thread_shared fn()`, and such a function holds no `shares` or `shares mut` value, whose count and value
## stay on one thread (E7015, as Rust's `Send`). rules.md section 3's line 1053 first message, in this
## program's lines (completeness G26's case, at the store).

class Counter:
    n: int = 0

class Bus:
    handlers: Array[thread_shared fn() changes Counter] = []

    fn on_event(owns f: thread_shared fn() changes Counter) changes handlers:
        this.handlers.push(f)

    fn fire() changes handlers, Counter:
        for ref mut h in this.handlers:
            h()

fn main() changes Counter:
    bus = Bus()
    count = shares mut Counter()
    bus.on_event(fn() with shares mut count changes Counter: count.n += 1)    #$ error[E7015]: `on_event` keeps functions any thread may run (`thread_shared fn() changes Counter`), and this one holds `count`, a `shares mut` owner, which stays on one thread; share the counter between threads: write `count = thread_shared Counter()` in place of line 24, give `Counter` the field `n: Atomic[int] = Atomic(0)` in place of line 10 (with `from std.sync import Atomic` at the top), and pass `fn() with thread_shared count changes Counter: count.n.fetch_add(1)`
    bus.fire()
    println(len(bus.handlers))
