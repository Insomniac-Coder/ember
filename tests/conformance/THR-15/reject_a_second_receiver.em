#$ test: compile-fail
#$ rules: THR-15
#$ error[E3040]: `rx` has been moved out of
# `[THR-15]` — a channel has one receiver: moving a `Receiver` gives it away.

from std.sync import channel

fn main():
    _tx, rx = channel[int](capacity=4)
    _first = rx
    _second = rx
