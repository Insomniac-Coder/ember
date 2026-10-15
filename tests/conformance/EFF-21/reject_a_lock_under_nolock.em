#$ test: compile-fail
#$ rules: EFF-21, EFF-18, EFF-5
#$ error[E4042]: `@nolock` function `bump` acquires a lock
#$ error[E4042]: `@nolock` function `try_bump` acquires a lock
#$ error[E4003]: `@noblock` function `wait` reaches an operation that may wait
# `[EFF-21]` — `@nolock` forbids `Lock`, including `try_lock`, which never blocks but still takes a
# lock; effects are independent (`[EFF-18]`): a blocking lock is also `Block`, `try_lock` is not.

from std.sync import Mutex

@nolock
fn bump(m: Mutex[int]):
    with g = m.lock():
        g += 1

@nolock
fn try_bump(m: Mutex[int]):
    match m.try_lock():
        Some(g):
            g += 1
        None:
            pass

@noblock
fn wait(m: Mutex[int]) -> int:
    with g = m.lock():
        return g

@noblock
fn try_read(m: Mutex[int]) -> int:
    match m.try_lock():
        Some(g):
            return g
        None:
            return 0

fn main():
    m = Mutex(1)
    bump(m)
    try_bump(m)
    println(wait(m), try_read(m))
