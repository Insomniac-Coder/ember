# Clause THR-20.1: "A `static mut` (`[STA-1]`) is reached only by the main thread and the tasks and
# jobs of the scopes it opens (...), unless it is `guarded by` a `static` `Lock` and used inside
# `with` that lock: a function that may run on another thread (...) and reads or changes it is
# `E7012`"; "So a function that may run on another thread and holds such a count counts as possibly
# running that value's `drop`, and the `drop` of everything it owns: (...) a module variable of the
# main thread they reach is `E7012`."
#$ test: compile-fail
#$ rules: THR-20, STA-1, RC-4
#$ error[E7012]: `tick` changes `FRAME` and `LOG`, module variables of the main thread, and this thread runs beside it; guard both with one lock: write `from std.sync import Lock` at the top, `static TICK_LOCK: Lock = Lock()`, then `static mut FRAME: int guarded by TICK_LOCK = 0` instead of line 17 and `static mut LOG: Array[String] guarded by TICK_LOCK = []` instead of line 18, and put `tick`'s body inside `with TICK_LOCK:`
#$ error[E7012]
# The spec's message, with this file's line numbers; a thread holding a count of a `thread_shared`
# value whose `drop` changes `LOG` is code only. Claimed for the file: the spec does not say whether
# `E7012` stands at the spawn or in the function (Q3).

import std.thread

static mut FRAME: int = 0
static mut LOG: Array[String] = []

class Ticket:
    n: int = 1

    fn drop():
        LOG.push(owns String("ticket"))

fn tick():
    FRAME += 1
    LOG.push(owns String("tick"))

fn main():
    h = thread.spawn(owns fn(): tick())
    h.join()
    t = thread_shared Ticket()
    k = thread.spawn(owns fn() thread_shared t: t.n)
    k.join()
