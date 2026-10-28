#$ test: compile-fail
#$ rules: R15, R12
## R12 (verdict 36): "a thread's function that sends, `fn() with owns tx changes Sender[int]:
## produce(tx)`".

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]) changes Sender[int]:
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main() changes Sender[int]:
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(fn() with owns tx: produce(tx))    #$ error[E2105]: this function calls `produce`, which changes `Sender[int]`; say so on its first line: `fn() with owns tx changes Sender[int]: produce(tx)`
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
