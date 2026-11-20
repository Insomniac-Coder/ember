# Clause THR-15.1: "`Sender` is `Clone` (`copy of tx` is another sender of the same channel)"; "**When
# the `Receiver` ends, so do the values still queued**: (...) so a `Sender` queued in its own channel
# (`tx.send(owns copy of tx)`) does not keep the channel alive."
#$ test: run-pass
#$ rules: THR-15, THR-10, THR-17
#$ stdout: 285
#$ stdout: 3
#$ stdout: done
# §XI.2's program; a second sender made with `copy of tx`; a sender queued in its own channel,
# which ends with the receiver.

import std.thread
from std.sync import Sender, channel

fn produce(tx: Sender[int]):
    for i in 0..10:
        tx.send(owns (i * i)).expect("receiver is gone")

fn main():
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(owns fn() owns tx: produce(tx))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
    tx2, rx2 = channel[int](capacity=4)
    other = copy of tx2
    tx2.send(owns 1).expect("receiver is gone")
    other.send(owns 2).expect("receiver is gone")
    println(rx2.recv().unwrap() + rx2.recv().unwrap())
    tx3, _rx3 = channel[Sender[int]](capacity=1)
    inner, _inner_rx = channel[int](capacity=1)
    tx3.send(owns copy of inner).expect("receiver is gone")
    println("done")
