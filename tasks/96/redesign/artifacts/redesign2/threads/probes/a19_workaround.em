# A19 workaround in Ember: an Atomic and a Mutex.
from std.sync import Atomic, Mutex

static FRAME: Atomic[int] = Atomic(0)
static LOG: Mutex[Array[String]] = Mutex([])

fn tick():
    n = FRAME.fetch_add(1) + 1
    with log = LOG.lock():
        log.push(f"tick {n}")

fn main():
    tick()
