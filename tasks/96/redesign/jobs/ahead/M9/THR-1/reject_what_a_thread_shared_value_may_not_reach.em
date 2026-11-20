# Clause THR-1.3: "its type is `Send` and `ThreadShared` (...): a one-thread owner, a `weakref T`
# link, a plain `fn()` value, or a part that is not `ThreadShared` (...) reachable from it is `E7015`
# at the line that makes it `thread_shared`, naming the field and the fix"; "**`thread_shared
# fn()`** (...) holds only `thread_shared` owners, values handed over to it and copies, never a
# one-thread owner or a `weakref T` link (`E7015`"
#$ test: compile-fail
#$ rules: THR-1, THR-8, THR-9, CLO-16
# The spec's three messages, with this file's line numbers.

from std.sync import RwLock

class FrameStats:
    draws: RwLock[Cell[int]] = RwLock(Cell(0))

class Bus:
    handlers: Array[fn()] = []

class Counter:
    n: int = 0

class Events:
    handlers: Array[thread_shared fn()] = []

    fn on_event(owns f: thread_shared fn()):
        this.handlers.push(owns f)

fn main():
    fs = thread_shared FrameStats()    #$ error[E7015]: `FrameStats.draws` is an `RwLock[Cell[int]]`, which several threads may not read at once, and a `thread_shared FrameStats` may be read from any thread; write `draws: Atomic[int] = Atomic(0)` in `FrameStats`
    b = thread_shared Bus()    #$ error[E7015]: `Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may run them on any thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`
    ev = Events()
    count = shares mut Counter()
    ev.on_event(owns fn() shares mut count: count.n += 1)    #$ error[E7015]: `on_event` keeps functions any thread may run (`thread_shared fn()`), and this one holds `count`, a `shares mut` owner, which stays on one thread; share the counter between threads: write `count = thread_shared Counter()` in place of line 31, give `Counter` the field `n: Atomic[int] = Atomic(0)` (with `from std.sync import Atomic` at the top), and pass `fn() thread_shared count: count.n.fetch_add(1)`
    println(fs.draws.read().get(), b.handlers.len())
