#$ test: compile-fail
#$ rules: THR-15
#$ error[E3040]: `rx` was handed over to `_first` on line 10 and is used on line 11; to keep both, write `_first = copy of rx` (a copy of your own) or `_first = shares rx` (two owners of one value)
# `[THR-15]` — a channel has one receiver: moving a `Receiver` gives it away.

from std.sync import channel

fn main():
    _tx, rx = channel[int](capacity=4)
    _first = rx
    _second = rx
