# Proposed syntax. The first attack's save of the lock field (its 3.3): a function that touches guarded
# fields without its own `with` gets "needs s.lock held", checked at each call. A function value made from
# it loses the precondition: a call through the value is not a call to `_bump`.
import std.thread
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int = 0

    fn _bump(self):
        self.words += 1             # summary: needs self.lock held

    fn add(self):
        with self.lock:
            self._bump()            # checked: holds the lock

fn main():
    s = Stats()
    bump = s._bump                  # a bound method as a value: its type is fn(), no precondition
    with scope = thread.scope():
        scope.spawn(fn(): s.add())
        scope.spawn(fn(): bump())   # runs _bump without the lock while the other task holds it
