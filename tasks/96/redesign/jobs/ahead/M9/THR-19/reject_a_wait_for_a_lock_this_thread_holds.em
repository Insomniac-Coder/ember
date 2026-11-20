# Clause THR-19.1: "While this thread holds a lock (...) these are `E7018` (...): a call through a
# function value (...) that may run a function taking a lock of the kind this thread holds (...); a
# join of a thread, or the end of a `with scope` block, whose task changes what the held lock
# guards; (...) and a second `read()` of an `RwLock` this thread holds `read()`"; "Inside a walk
# under that lock, such a call is `E7018`, not `E3020`"
#$ test: compile-fail
#$ rules: THR-19, THR-5, THR-10, THR-18, DIA-28
#$ error[E7018]
# The spec's four messages, with this file's line numbers; the scope that ends inside the lock its
# task needs (`[THR-5]`) is code only, claimed for the file (Q3).

import std.thread
from std.sync import Lock, RwLock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(owns String(s))

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int):
    with RESULTS_LOCK:
        RESULTS.push(owns x)

fn through_a_function_value(log: Log):
    note = fn(): log.push("x")
    with log.lock:
        note()    #$ error[E7018]: `note()` may run the function stored on line 31, which takes a `Log`'s lock (`log.push`, under `lock`), and this thread holds `log`'s lock, so taking it again would wait for ever; call `note()` after the `with log.lock:` block: move line 33 below the block

fn in_a_walk(log: Log):
    note = fn(): log.push("y")
    with log.lock:
        for line in log.lines:
            println(line)
            note()    #$ error[E7018]: this loop walks `log.lines` (line 38) under `log`'s lock, and `note()` (line 40) may run the function stored on line 36, which takes a `Log`'s lock, so it would wait for the lock this thread holds; call `note()` after the `with log.lock:` block: move line 40 below the block

fn a_join():
    with RESULTS_LOCK:
        t = thread.spawn(owns fn(): worker(2))
        t.join()    #$ error[E7018]: `t.join()` (line 45) waits for the thread started on line 44, whose function changes `RESULTS` under `RESULTS_LOCK`, and this thread holds that lock, so each would wait for the other for ever; join after the `with RESULTS_LOCK:` block: move line 45 below the block

fn a_scope():
    with RESULTS_LOCK:
        with scope = thread.scope():
            scope.spawn(owns fn(): worker(3))

fn two_reads():
    rw = RwLock([1])
    with g = rw.read():
        println(g.len())
        with h = rw.read():    #$ error[E7018]: `rw.read()` (line 56) takes a read lock this thread already holds (line 54), and a writer arriving between them would make each wait for the other for ever; use the guard `g` already held, or take the second read after the first ends
            println(h.len())

fn main():
    log = Log()
    through_a_function_value(log)
    in_a_walk(log)
    a_join()
    a_scope()
    two_reads()
