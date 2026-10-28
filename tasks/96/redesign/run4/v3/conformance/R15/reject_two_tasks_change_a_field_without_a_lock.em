#$ test: compile-fail
#$ rules: R15
## R15: "E7007 for two of them changing one place other than by a reduction or under a lock"
## (section 3's E7007 message, in this program's lines; AN-74).

import std.thread

class Stats:
    total: int = 0

fn count(texts: Array[String]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() changes stats: stats.total += t.len())    #$ error[E7007]: two tasks change `stats.total` (line 15), each without a lock; count it with an atomic, `total: Atomic[int] = Atomic(0)` in `Stats` and `stats.total.fetch_add(t.len())` on line 15, or guard it: in `Stats` write `lock: Lock = Lock()`, `total: int guarded by lock = 0` and `fn add_total(n: int) changes total:` with `with this.lock: this.total += n` as its body, and call `stats.add_total(t.len())` on line 15
    return stats.total

fn main():
    println(count([String("ab"), String("c")]))
