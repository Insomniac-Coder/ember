#$ test: run-pass
#$ rules: THR-20, STA-1
#$ stdout: 21
#$ 1
# `[THR-20]` — a thread whose function reaches no module variable is no `E7012`; the tasks of a scope the
# main thread opens reach its module variables, as the main thread does.

import std.thread

static mut FRAME: int = 0

fn twenty_one() -> int:
    return 21

fn tick():
    FRAME += 1

fn main():
    h = thread.spawn(owns twenty_one)
    println(h.join())
    with s = thread.scope():
        s.spawn(owns tick)
    println(FRAME)
