# Clause THR-11.1: "What each task changes and reads is worked out from its body and every function
# it may run (`[EFF-24]`), and is checked against the other tasks and the parent until the task's
# join (...): two of them changing one place, or one reading a place another changes, is `E7007`";
# "A `Receiver` is one consumer: it is used by one task, or by the parent, until the join, as any
# other place (`E7007`"
#$ test: compile-fail
#$ rules: THR-11, THR-13, THR-15, EFF-24
#$ error[E7007]
#$ error[E7007]
#$ error[E7007]: two tasks take values out of `rx`, which one consumer owns; give each task a channel of its own, or let one task receive and hand the values on
# Two tasks change one place (through a callee); one reads what another changes; two tasks receive
# from one `Receiver` (the spec's message). Claimed for the file: the spec does not say on which
# line `E7007` stands (Q3).

import std.thread
from std.sync import channel

fn set_first(ref mut xs: Array[int], v: int):
    xs[0] = v

fn main():
    counts = [0, 0]
    with scope = thread.scope():
        scope.spawn(owns fn(): set_first(ref mut counts, 1))
        scope.spawn(owns fn(): set_first(ref mut counts, 2))
    xs = [1]
    with scope = thread.scope():
        scope.spawn(owns fn(): xs.push(owns 2))
        scope.spawn(owns fn(): println(xs.len()))
    tx, rx = channel[int](capacity=4)
    tx.send(owns 1).expect("receiver is gone")
    tx.send(owns 2).expect("receiver is gone")
    with scope = thread.scope():
        scope.spawn(owns fn(): println(rx.recv().unwrap()))
        scope.spawn(owns fn(): println(rx.recv().unwrap()))
    println(counts[0])
