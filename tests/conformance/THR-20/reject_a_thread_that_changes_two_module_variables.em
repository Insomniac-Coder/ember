#$ test: compile-fail
#$ rules: THR-20, STA-1
# `[THR-20]` — a function another thread may run that changes module variables of the main thread is
# `E7012`, with the spec's text: the lock to declare, the declarations to write, where its body goes.

import std.thread

static mut FRAME: int = 0
static mut LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push(owns String("tick"))

fn main():
    h = thread.spawn(owns tick)    #$ error[E7012]: `tick` changes `FRAME` and `LOG`, module variables of the main thread, and this thread runs beside it; guard both with one lock: write `from std.sync import Lock` at the top, `static TICK_LOCK: Lock = Lock()`, then `static mut FRAME: int guarded by TICK_LOCK = 0` instead of line 8 and `static mut LOG: Array[String] guarded by TICK_LOCK = []` instead of line 9, and put `tick`'s body inside `with TICK_LOCK:`
    h.join()
