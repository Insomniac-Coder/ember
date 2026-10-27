#$ test: compile-fail
#$ rules: R15, R12, R5
## R12 and R15 (verdicts 30, 36): `fn main() changes Sender[int]:`, "which starts that thread".

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]) changes Sender[int]:
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main():    #$ error[E2105]: `main` starts a thread whose function changes `Sender[int]` (`fn() with owns tx changes Sender[int]: produce(tx)`, line 14); add it to `main`'s first line: `fn main() changes Sender[int]:`
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(fn() with owns tx changes Sender[int]: produce(tx))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
