#$ test: run-pass
#$ rules: THR-15, THR-10, THR-8
#$ profiles: debug, development
#$ stdout: 10000 100000
#$ true true
#$ full, kept 3 false
#$ Some(1) None
#$ gone, kept 9 true
#$ 499500
# `[THR-15]` — four producers (`clone`s of one `Sender`) fill a channel of two slots, so they wait while it
# is full; the loop ends once every sender is gone. `try_send` gives the value back when the queue is
# full, `send` when the receiver is gone; `try_recv` never waits; an unbounded channel grows.

import std.thread
from std.sync import Sender, channel, channel_unbounded

fn produce(tx: Sender[String], who: int, n: int):
    for i in range(n):
        tx.send("p{who}-{i}").expect("receiver is gone")

fn main():
    tx, rx = channel[String](capacity=2)
    workers: Array[thread.JoinHandle[void]] = []
    for who in range(4):
        mine = tx.clone()
        workers.push(thread.spawn(owned fn() => produce(mine, who, 2500)))
    mem.drop(tx)
    count = 0
    chars = 0
    for v in rx:
        count += 1
        chars += v.len()
    while workers.len() > 0:
        match workers.pop():
            Some(w):
                w.join()
            None:
                pass
    println(count, chars)
    small_tx, small_rx = channel[int](capacity=1)
    println(small_tx.try_send(1).is_ok(), small_tx.try_send(2).is_err())
    match small_tx.try_send(3):
        Ok(_):
            println("sent")
        Err(e):
            println("full, kept", e.value, e.disconnected)
    println(small_rx.try_recv(), small_rx.try_recv())
    mem.drop(small_rx)
    match small_tx.send(9):
        Ok(_):
            println("sent")
        Err(e):
            println("gone, kept", e.value, e.disconnected)
    utx, urx = channel_unbounded[int]()
    for i in range(1000):
        utx.send(i).expect("open")
    mem.drop(utx)
    s = 0
    for v in urx:
        s += v
    println(s)
