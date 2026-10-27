#$ test: run-pass
#$ rules: R15, R12, R5, R4
#$ stdout: 285
## R5 and R12 (verdict 86): a send, and taking values out through an end a function was given, are
## worked out from the body like every change, and no first line says them (the words verdicts 36 and
## 37 asked for are gone with verdict 86); the ends take no word at a call (R4).

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]):
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn consume(rx: Receiver[int]) -> int:
    total = 0
    for v in rx:
        total += v
    return total

fn main():
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(fn() owns tx: produce(tx))
    consumer = thread.spawn(fn() owns rx -> int: consume(rx))
    producer.join()
    println(consumer.join())
