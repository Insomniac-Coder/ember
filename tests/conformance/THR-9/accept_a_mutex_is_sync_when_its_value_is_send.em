#$ test: run-pass
#$ rules: THR-9, THR-1, THR-3
#$ stdout: ['#x']
# `[THR-9]` — `Mutex[T]` for a `Send` `T` is `Sync`: one thread at a time holds its `ref mut T`, so a
# `@sync` class may hold one, and mutate what it guards after `init`.

from std.sync import Mutex

@sync
class Stats:
    tags: Mutex[Array[String]] = Mutex([])

fn main():
    stats = Stats()
    with tags = stats.tags.lock():
        tags.push("#x".to_string())
    with tags = stats.tags.lock():
        println(tags)
