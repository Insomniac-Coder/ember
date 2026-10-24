# Proposed syntax, speed attack on the first attack's rule 2 of its 3.1 ("Everything stored into a lock is
# flagged at the store. A Mutex's value or a lock field, whether or not the object holding it has been
# shared."). A cache class with a lock field, used by one thread only (a library class written to be safe
# for threads, as Python's logging.Handler is). Every Entry stored into it is walked and flagged, and from
# then on every count change on it is atomic, in a program that never starts a thread.
from std.sync import Lock

class Entry:
    key: String = ""
    hits: int = 0

class Cache:
    lock: Lock = Lock()
    entries: Array[Entry] = []

    fn put(self, e: Entry):
        with self.lock:
            self.entries.push(e)        # flagged here (first attack's rule), though no thread exists

fn main():
    c = Cache()
    for i in 0..1000000:
        e = Entry(key=f"k{i}")
        c.put(e)
        e.hits += 1                     # and the shared-object rule now refuses this write (4.6)
