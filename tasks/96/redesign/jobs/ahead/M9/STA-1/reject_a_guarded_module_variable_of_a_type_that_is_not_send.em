# Clause STA-1.1: "A `guarded by` `static mut` is a `Mutex[T]` written as a module variable
# (`[THR-18]`), so its `T` MUST be `Send` (`E7002`"; "`T` MUST be `ThreadShared` (`E7002`)" for a
# `static`
#$ test: compile-fail
#$ rules: STA-1, THR-8, THR-9, THR-20
# The spec's message; a `static` holding a `Receiver`, which is not `ThreadShared` (`[THR-20]`), is
# code only.

from std.sync import Lock, Receiver, channel

class Table:
    n: int = 0

static L: Lock = Lock()
static mut CACHE: shares mut Table guarded by L = shares mut Table()    #$ error[E7002]: `CACHE` is guarded by `L`, so any thread may reach it under the lock, and a `shares mut Table` stays on one thread; share it between threads: write `static mut CACHE: thread_shared Table guarded by L = thread_shared Table()`

fn make_rx() -> Receiver[int]:
    _tx, rx = channel[int](capacity=1)
    return rx

static INBOX: Receiver[int] = make_rx()    #$ error[E7002]

fn main():
    with L:
        println(CACHE.n)
