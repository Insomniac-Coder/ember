# PROPOSED syntax. One class object reachable from two Mutexes, written under each lock.
import std.thread
from std.sync import Mutex

class Player:
    hp: int = 10

fn main():
    p = Player()
    a: Mutex[Array[Player]] = Mutex([p])
    b: Mutex[Array[Player]] = Mutex([p])
    t1 = thread.spawn(fn():
        with g = a.lock():
            g[0].hp -= 1)                # "under a Mutex"
    t2 = thread.spawn(fn():
        with g = b.lock():
            g[0].hp -= 1)                # under another Mutex: the same field, no common lock
    t1.join(); t2.join()
