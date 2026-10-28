#$ test: compile-fail
#$ rules: R15, R12, R5
## R12 (verdict 37): "so is taking values out through a receiving end a function was given (`rx.recv()`,
## `for v in rx:`): `fn consume(rx: Receiver[int]) changes Receiver[int] -> int:`".

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]) changes Sender[int]:
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn consume(rx: Receiver[int]) -> int:    #$ error[E2105]: `consume` takes values out of `rx` (`for v in rx:`, a change of the channel); add it to `consume`'s first line: `fn consume(rx: Receiver[int]) changes Receiver[int] -> int:`
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
