#$ test: compile-fail
#$ rules: R15, R12, R5
## R12 (verdict 36): "A send on a channel end is a change of the channel, named by the type of the end"
## (the message is written to R18; the rules give none for this case).

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]):    #$ error[E2105]: `produce` sends on `tx` (`tx.send(i * i)`, a change of the channel); add it to `produce`'s first line: `fn produce(tx: Sender[int]) changes Sender[int]:`
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
