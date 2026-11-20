# Clause THR-20.1: "A `static` of a type that changes only through itself (`Lock`, `Atomic`,
# `Mutex`, `RwLock`, a `Sender` (...)) is usable from any thread"; a `static mut` reached by "the
# tasks and jobs of the scopes it opens"; and one `guarded by` a static `Lock`
#$ test: run-pass
#$ rules: THR-20, THR-21, STA-1
#$ stdout: 3
#$ stdout: 5
#$ stdout: 1
# A thread adds to a `static` `Atomic`; scoped tasks add to an unguarded `static mut` (a
# reduction); a thread changes a guarded one under its lock.

import std.thread
from std.sync import Atomic, Lock

static HITS: Atomic[int] = Atomic(0)
static mut TOTAL: int = 0
static FRAME_LOCK: Lock = Lock()
static mut FRAME: int guarded by FRAME_LOCK = 0

fn bump_frame():
    with FRAME_LOCK:
        FRAME += 1

fn main():
    h = thread.spawn(owns fn(): HITS.fetch_add(3))
    h.join()
    println(HITS.load())
    with scope = thread.scope():
        scope.spawn(owns fn(): TOTAL += 2)
        scope.spawn(owns fn(): TOTAL += 3)
    println(TOTAL)
    k = thread.spawn(owns fn(): bump_frame())
    k.join()
    with FRAME_LOCK:
        println(FRAME)
