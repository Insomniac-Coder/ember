# PROPOSED syntax. _bump touches a guarded field without its own `with`; every caller holds the lock.
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int = 0

    fn _bump(self):
        self.words += 1            # summary: "unguarded write of Stats.words" -> Stats may not be shared

    fn add(self):
        with self.lock:
            self._bump()
