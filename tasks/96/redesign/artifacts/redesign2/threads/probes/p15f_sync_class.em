# A15: what sharing a counter object with an unscoped thread takes today.
import std.thread
from std.sync import Atomic, Mutex

@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    tags: Mutex[Array[String]] = Mutex([])

class PlainStats:
    words: int = 0

fn main():
    stats = Stats()
    h = thread.spawn(owned fn(): stats.words.fetch_add(1))
    h.join()
    ps = PlainStats()
    h2 = thread.spawn(owned fn(): ps.words += 1)
    h2.join()
    println(stats.words.load())
