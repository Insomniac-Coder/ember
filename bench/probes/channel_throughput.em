# Send a million numbers from one thread to another through a channel of 64 slots and add them up
# (`[THR-15]`): the cost of `send`, `recv` and the waits a full or empty queue makes. The C twin is
# the same bounded queue over the platform's lock and condition variables.

import std.thread
from std.sync import Sender, channel

fn produce(tx: Sender[int], n: int):
    for i in range(n):
        tx.send(i % 1000).expect("receiver is gone")

fn main():
    n = 1_000_000
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(owned fn() => produce(tx, n))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
