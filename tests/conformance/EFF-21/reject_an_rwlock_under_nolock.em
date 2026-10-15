#$ test: compile-fail
#$ rules: EFF-21, EFF-5
#$ error[E4042]: `@nolock` function `peek` acquires a lock
#$ error[E4042]: `@nolock` function `try_poke` acquires a lock
# `[EFF-21]` — `RwLock.read` and `try_write` take a lock, so `@nolock` refuses both.

from std.sync import RwLock

@nolock
fn peek(rw: RwLock[int]) -> int:
    with r = rw.read():
        return r

@nolock
fn try_poke(rw: RwLock[int]):
    match rw.try_write():
        Some(w):
            w += 1
        None:
            pass

fn main():
    rw = RwLock(1)
    try_poke(rw)
    println(peek(rw))
