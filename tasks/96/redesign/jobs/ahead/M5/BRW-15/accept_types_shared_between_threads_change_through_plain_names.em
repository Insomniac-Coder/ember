# Clause BRW-15.3: "The types made to be shared between threads are used through any name, a plain
# one included, by their own operations: a `Mutex`, `RwLock`, `Atomic` or channel end, and the
# `guarded by` fields of an object under their lock (...). Each orders its own change (...), and
# handed to a call it takes no word (`[FN-2a]`, Part XI)."
#$ test: run-pass
#$ rules: BRW-15, FN-1, FN-2a, THR-3
#$ stdout: 5 2 1 7
# Each is changed through a plain parameter, and handed to the call with no word.
from std.sync import Atomic, Mutex, Lock, Sender, channel

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

fn add(total: Atomic[int], n: int):
    total.fetch_add(n)

fn put(m: Mutex[Array[int]], v: int):
    with g = m.lock():
        g.push(owns v)

fn note(log: Log):
    with log.lock:
        log.lines.push(owns String("seen"))

fn send(tx: Sender[int]):
    tx.send(owns 7).expect("receiver is gone")

fn main():
    total = Atomic(0)
    add(total, 5)
    m = Mutex([1])
    put(m, 2)
    log = Log()
    note(log)
    tx, rx = channel[int](capacity=4)
    send(tx)
    got = rx.recv()
    count = 0
    with g = m.lock():
        count = g.len()
    lines = 0
    with log.lock:
        lines = log.lines.len()
    println(total.load(), count, lines, got.unwrap())
