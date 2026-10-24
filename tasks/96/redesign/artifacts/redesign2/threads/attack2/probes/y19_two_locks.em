# Proposed syntax, readability attack. A class with two locks, each guarding its own fields (a common
# Python shape: one lock for statistics, one for a cache, so the two never wait for each other). The
# lock-field rule (answers report 4.5) speaks of "a class with one field of type Lock"; with two it does
# not say which fields each guards.
from std.sync import Lock

class Server:
    stats_lock: Lock = Lock()
    hits: int = 0
    cache_lock: Lock = Lock()
    cache: Map[String, String] = {}

    fn hit(self):
        with self.stats_lock:
            self.hits += 1

    fn remember(self, k: String, v: String):
        with self.cache_lock:
            self.cache[k] = v
