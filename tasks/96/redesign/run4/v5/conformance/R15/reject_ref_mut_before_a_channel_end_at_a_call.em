#$ test: compile-fail
#$ rules: FN-2a
## design rules: R15, R4
## R4 (verdict 3): "An argument given to a `Mutex`, `RwLock` or `Atomic` parameter, or a channel end,
## takes no word" (the message follows section 3's `Mutex` one). The nearest allowed program is
## `accept_main_receives_through_its_own_end_while_a_thread_sends.em`.

import std.thread
from std.sync import Receiver, Sender, channel

fn produce(tx: Sender[int]):
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main():
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(fn() owns tx: produce(ref mut tx))    #$ error[E3028]: `produce` sends through `tx` by the channel's own operation; a channel end takes no word at the call: `produce(tx)`
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
