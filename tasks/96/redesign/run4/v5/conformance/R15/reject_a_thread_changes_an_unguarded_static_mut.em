#$ test: compile-fail
#$ rules: THR-20
## design rules: R15, R5
## R15: "`static mut NAME: T = e` ... is reached only by the main thread and its scoped tasks unless it
## is `guarded by` a `static` lock (E7012)" (section 3's message, in this program's lines; AN-76).

import std.thread

static mut FRAME: int = 0
static mut LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push(String("tick"))

fn main():
    a = thread.spawn(fn(): tick())    #$ error[E7012]: `tick` changes `FRAME` and `LOG`, module variables of the main thread, and this thread runs beside it; guard both with one lock: write `from std.sync import Lock` at the top, `static TICK_LOCK: Lock = Lock()`, then `static mut FRAME: int guarded by TICK_LOCK = 0` instead of line 8 and `static mut LOG: Array[String] guarded by TICK_LOCK = []` instead of line 9, and put `tick`'s body inside `with TICK_LOCK:`
    a.join()
