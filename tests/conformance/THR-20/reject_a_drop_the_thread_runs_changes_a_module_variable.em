#$ test: compile-fail
#$ rules: THR-20, DRP-8, STA-1
# `[THR-20]`, `[DRP-8]` — a value the thread's function ends runs its `drop` on that thread: a `drop` that
# changes a module variable of the main thread is `E7012`, the method named by its type.

import std.thread

static mut LOG: Array[String] = []

class Ticket:
    n: int = 1

    fn drop():
        LOG.push(owns String("ticket"))

fn work():
    t = Ticket()
    println(t.n)

fn main():
    h = thread.spawn(owns work)    #$ error[E7012]: `Ticket.drop` changes `LOG`, a module variable of the main thread, and this thread runs beside it; guard it with one lock: write `from std.sync import Lock` at the top, `static TICKET_LOCK: Lock = Lock()`, then `static mut LOG: Array[String] guarded by TICKET_LOCK = []` instead of line 8, and put `Ticket.drop`'s body inside `with TICKET_LOCK:`
    h.join()
