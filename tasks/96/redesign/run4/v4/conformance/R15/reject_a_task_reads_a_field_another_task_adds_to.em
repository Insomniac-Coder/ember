#$ test: compile-fail
#$ rules: R15, R12
## R15 (verdict 77): "A task that reads the current value (`if stats.total < 10: stats.total += x`) ...
## still needs an `Atomic` or a lock (E7007)" (section 3's E7007 message, in this program's lines; the
## task is a local function, verdict 51).

import std.thread

class Stats:
    total: int = 0

fn add_small(xs: Array[int]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for x in xs:
            fn add() with copy of x changes stats:
                if stats.total < 10:
                    stats.total += x    #$ error[E7007]: two tasks change `stats.total` (line 18), and each reads it first (`if stats.total < 10:`, line 17), so each may see a total the other is changing; count it with an atomic: write `from std.sync import Atomic` at the top, `total: Atomic[int] = Atomic(0)` instead of line 10, `if stats.total.load() < 10:` on line 17, `stats.total.fetch_add(x)` on line 18 and `return stats.total.load()` on line 20; or guard it: write `from std.sync import Lock` at the top, in `Stats` `lock: Lock = Lock()`, `total: int guarded by lock = 0` and `fn add_below(limit: int, n: int) changes total:` whose body is `with this.lock:`, inside it `if this.total < limit:` and inside that `this.total += n`, call `stats.add_below(10, x)` in place of lines 17 and 18, and read it under the lock: `with stats.lock:` and inside it `return stats.total` in place of line 20
            scope.spawn(add)
    return stats.total

fn main():
    println(add_small([1, 2]))
