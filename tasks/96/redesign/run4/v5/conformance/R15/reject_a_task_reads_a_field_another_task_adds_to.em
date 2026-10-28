#$ test: compile-fail
#$ rules: THR-11, THR-21
## design rules: R15, R12
## R15 (verdict 77): "a task that reads the current value (`if stats.total < 10: stats.total += x`) ...
## still need[s] a lock (E7007)" (section 3's E7007 message, which gives only the lock, which keeps the
## read and the add together: F-10; in this program's lines; the task is a local function, verdict 51).
## The nearest allowed program is `accept_a_task_reads_and_adds_to_a_guarded_field_under_its_lock.em`.

import std.thread

class Stats:
    total: int = 0

fn add_small(xs: Array[int]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for x in xs:
            fn add() copy of x:
                if stats.total < 10:
                    stats.total += x    #$ error[E7007]: two tasks change `stats.total` (line 19), and each reads it first (`if stats.total < 10:`, line 18), so each may see a total the other is changing; guard it, which keeps the read and the add together: write `from std.sync import Lock` below line 8; in `Stats`, `lock: Lock = Lock()` above line 11, `total: int guarded by lock = 0` in place of line 11, and `fn add_below(limit: int, n: int):` whose body is `with this.lock:`, inside it `if this.total < limit:` and inside that `this.total += n`; call `stats.add_below(10, x)` in place of lines 18 and 19; and read it under the lock: `with stats.lock:` and inside it `return stats.total`, in place of line 21
            scope.spawn(add)
    return stats.total

fn main():
    println(add_small([1, 2]))
