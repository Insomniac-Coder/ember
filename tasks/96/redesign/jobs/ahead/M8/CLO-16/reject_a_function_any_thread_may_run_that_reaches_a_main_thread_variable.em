# Clause CLO-16.2: "A function that captures nothing, or only `thread_shared` values and copies of
# values that may go to another thread, meets `thread_shared fn()` only if, through its chain of calls
# (`[EFF-24]`), it reaches no `static mut` that is not `guarded by` a `static` `Lock` and no
# one-thread value by any route; the check is made where the function is given to, stored into or
# coerced to such a type"
#$ test: compile-fail
#$ rules: CLO-16, THR-20, EFF-24
# The spec's message, with this file's line number. (CLO-16's `E7015` messages are in
# `ahead/M9/THR-1`.)

class Bus:
    handlers: Array[thread_shared fn()] = []

static mut FRAME: int = 0

fn on_frame():
    FRAME += 1

fn main():
    bus = Bus()
    bus.handlers.push(owns on_frame)    #$ error[E7012]: `on_frame` changes `FRAME`, a module variable of the main thread, and `Bus.handlers` holds functions any thread may run (`thread_shared fn()`); guard it: write `static TICK_LOCK: Lock = Lock()` and `static mut FRAME: int guarded by TICK_LOCK = 0` instead of line 14, and put the change inside `with TICK_LOCK:`
