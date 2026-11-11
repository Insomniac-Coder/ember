#$ test: run-pass
#$ rules: EFF-18, EFF-21
#$ stdout: 0
# `[EFF-18]` — `try_lock` has `Sync + Lock` and no `Block`: `@noblock` and `@noio` accept it.

from std.sync import Mutex

@noblock
@noio
fn peek(m: Mutex[int]) -> int:
    match m.try_lock():
        Some(g):
            return g
        None:
            return 0

fn main():
    println(peek(Mutex(0)))
