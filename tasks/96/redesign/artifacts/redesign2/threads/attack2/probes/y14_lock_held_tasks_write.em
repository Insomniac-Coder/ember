# Proposed syntax (attack program; today's compiler cannot check it: there is no `Lock`, E1061).
# The parent takes the lock; tasks spawned inside the `with` write guarded fields. Every write is
# "lexically inside `with s.lock:` on the same handle" (the lock-field rule, answers report 4.5), and
# writes to lock-guarded fields are "not conflicts" (scope rule, rule 3 of 4.3). But the lock is held by
# the parent thread, and the tasks run at the same time on other threads: a data race on `words`, and on
# `tags`, whose `push` may reallocate the buffer another task is writing into.
import std.thread
from std.sync import Lock, Mutex

class Stats:
    lock: Lock = Lock()
    words: int = 0
    tags: Array[String] = []

fn count(stats: Stats, texts: Array[String]):
    with stats.lock:
        with scope = thread.scope():
            for t in texts:
                scope.spawn(fn():
                    for w in t.split_whitespace():
                        stats.words += 1
                        if w.starts_with("#"):
                            stats.tags.push(w.to_string()))

# The same through a Mutex guard, under the first attack's saved rule (its 3.4): "writes through a Mutex
# guard ... are allowed when every access to that field in the scope goes through the same Mutex".
# Today this is E7004 ("a MutexGuard is not Send", probe x05); the proposal removes Send.
fn fill(m: Mutex[Array[int]]):
    with g = m.lock():
        with scope = thread.scope():
            scope.spawn(fn(): g.push(1))
            scope.spawn(fn(): g.push(2))
