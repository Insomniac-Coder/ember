#$ test: run-pass
#$ rules: THR-20, STA-1
#$ stdout: 21
#$ 1
# `[THR-20]` — a thread whose function reaches no module variable is no `E7012`, and the main thread reaches
# its module variables as it likes. (The tasks of a scope, which reach them too under `[THR-11]`'s check, wait
# for M9: `ahead/M9/THR-20/accept_what_threads_and_tasks_may_reach.em`.)

import std.thread

static mut FRAME: int = 0

fn twenty_one() -> int:
    return 21

fn main():
    h = thread.spawn(owns twenty_one)
    println(h.join())
    FRAME += 1
    println(FRAME)
