#$ test: compile-fail
#$ rules: THR-15
#$ error[E1010]: `std.sync.Receiver[i64]` has no method named `clone`
# `[THR-15]` — a `Sender` is `Clone`; a `Receiver` is not, so a channel keeps one consumer.

from std.sync import channel

fn main():
    tx, rx = channel[int](capacity=4)
    more = tx.clone()
    other = rx.clone()
