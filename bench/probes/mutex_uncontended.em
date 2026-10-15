# Lock and unlock a `Mutex` no other thread wants, 50 million times, adding under the lock: the
# cost of `with g = m.lock():` when it never waits (`[THR-3]`). The C twin uses the platform's
# own lightweight lock (SRWLOCK on Windows, a pthread mutex elsewhere).

from std.sync import Mutex

fn main():
    m = Mutex(0)
    n = 50_000_000
    for i in range(n):
        with g = m.lock():
            g += i % 7
    with g = m.lock():
        println(g)
