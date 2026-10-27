#$ test: run-pass
#$ rules: THR-1, CLO-1, FN-4
#$ stdout: 285
## design rules: R15, R12, R5
## R5 and R12 (verdict 86): `main` receives through its own `rx` while the thread it starts sends; no
## first line says what either does to the channel (verdicts 36 and 37 asked for a word on the sending
## side). Step 3's `accept_an_own_local_end_needs_no_word.em`, renamed: no end needs a word now.

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]):
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main():
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(fn() owns tx: produce(tx))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
