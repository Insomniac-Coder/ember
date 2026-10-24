# A15: the closest spelling today: @sync, every mutable field wrapped (Atomic, Mutex).
import std.thread
from std.sync import Atomic, Mutex

@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    tags: Mutex[Array[String]] = Mutex([])

    fn add(self, w: str):
        self.words.fetch_add(1)
        if w.starts_with("#"):
            with tags = self.tags.lock():
                tags.push(w.to_string())

fn count_into(stats: Stats, t: ref String):
    for w in t.split_whitespace():
        stats.add(w)

fn main():
    stats = Stats()
    texts = ["#ember is fast".to_string(), "no tags".to_string()]
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() => count_into(stats, t))
    println(stats.words.load())
