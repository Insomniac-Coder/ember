#$ test: run-pass
#$ rules: THR-15, THR-10
#$ profiles: debug, development
#$ stdout: 285
# `[THR-15]` — Part XI.2's example: a producer thread sends the squares of 0..10 through a bounded
# channel; the receiving loop ends at `None`, once the only `Sender` has been dropped with the thread's
# function.

import std.thread
from std.sync import Sender, channel

fn produce(tx: Sender[int]):
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main():
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(owned fn() => produce(tx))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
