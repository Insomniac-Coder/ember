#$ test: run-pass
#$ rules: R15, R12
#$ stdout: 285
## R12 (verdict 37): "A function's own local end needs no word, as any own local": `main` receives
## through its own `rx` and names only what the thread it starts changes (verdict 36).

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]) changes Sender[int]:
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main() changes Sender[int]:
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(fn() with owns tx changes Sender[int]: produce(tx))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
