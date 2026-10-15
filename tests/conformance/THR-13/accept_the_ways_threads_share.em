#$ test: run-pass
#$ rules: THR-13, THR-1, STA-1, HEAP-10, THR-11
#$ stdout: 3 5 7 [1, 2] 9 moved
# `[THR-13]` — two threads reach one location only through (a) a `@sync` object, (b) a `static`, (c)
# a `SyncShared`, (d) a borrow handed to a scoped task (shared of a `Sync` type, or exclusive), or
# (e) a value moved to the other thread; each is shown here.

import std.thread
from std.sync import Atomic, SyncShared

@sync
class Stats:
    seen: Atomic[int] = Atomic(0)

static TOTAL: Atomic[int] = Atomic(0)

fn bump(mut s: MutSpan[int], at: int, by: int):
    s[at] += by

fn main():
    stats = Stats()
    theirs = stats
    a = thread.spawn(owned fn() => theirs.seen.fetch_add(3))
    a.join()
    b = thread.spawn(owned fn() => TOTAL.fetch_add(5))
    b.join()
    shared = SyncShared(Atomic(7))
    copy = shared
    c = thread.spawn(owned fn() => copy.get().load())
    seven = c.join()
    counts = [0, 0]
    first, second = counts.as_mut_span().split_at(1)
    nine = Atomic(9)
    with scope = thread.scope():
        scope.spawn(fn() => bump(first, 0, 1))
        scope.spawn(fn() => bump(second, 0, 2))
        scope.spawn(fn() => nine.load())
    text = "moved".to_string()
    d = thread.spawn(owned fn() => text.clone())
    println(stats.seen.load(), TOTAL.load(), seven, counts, nine.load(), d.join())
