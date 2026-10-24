# A15 proposed: Python's spelling of a shared object, a lock field that guards the other fields.
import std.thread
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int = 0
    tags: Array[String] = []

    fn add(self, w: str):
        with self.lock:
            self.words += 1
            if w.starts_with("#"):
                self.tags.push(w.to_string())

fn count_into(stats: Stats, t: ref String):
    for w in t.split_whitespace():
        stats.add(w)

fn main():
    stats = Stats()
    texts = ["#ember is fast".to_string(), "no tags".to_string()]
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() => count_into(stats, t))
    println(stats.words)
