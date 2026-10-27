#$ test: run-pass
#$ rules: R15, R12, R5
#$ stdout: 285
## R12 (verdicts 36, 37): a send names `changes Sender[int]`, taking values out through an end a
## function was given names `changes Receiver[int]`; `main`, which starts both threads, names both
## (verdict 30). The ends take no word at a call (R4).

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]) changes Sender[int]:
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn consume(rx: Receiver[int]) changes Receiver[int] -> int:
    total = 0
    for v in rx:
        total += v
    return total

fn main() changes Sender[int], Receiver[int]:
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(fn() with owns tx changes Sender[int]: produce(tx))
    consumer = thread.spawn(fn() with owns rx changes Receiver[int] -> int: consume(rx))
    producer.join()
    println(consumer.join())
