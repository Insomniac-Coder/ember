#$ test: run-pass
#$ rules: THR-1, FN-4, CLO-1
#$ stdout: 3
## design rules: R15, R5, R12
## R15: two tasks changing one place "under a lock" is not E7007; `guarded by lock` fields are changed
## inside `with this.lock:` (AN-74's NEW `Stats` and `count`, with the loop variable's copy of verdict
## 65).

import std.thread
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0
    tags: Array[String] guarded by lock = []

    fn add_all(text: StrView):
        for w in text.split_whitespace():
            with this.lock:
                this.words += 1
                if w.starts_with("#"):
                    this.tags.push(String(w))

fn count(texts: Array[String]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() copy of t: stats.add_all(t))
    with stats.lock:
        return stats.words

fn main():
    println(count([String("a #b"), String("c")]))
