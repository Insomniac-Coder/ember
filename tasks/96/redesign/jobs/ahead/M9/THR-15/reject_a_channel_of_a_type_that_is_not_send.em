# Clause THR-15.1: "A channel carries its values to another thread, so `T` MUST be `Send` (`[THR-8]`):
# `channel[T]` or `channel_unbounded[T]` for a `T` that is not is `E7015` at the line that makes the
# channel, naming what keeps `T` on one thread"
#$ test: compile-fail
#$ rules: THR-15, THR-8, LT-46
# The spec's message (verdict 164).

from std.sync import channel

class Player:
    hp: int = 5

class Watcher:
    target: sharedref mut Player

fn main():
    _tx, rx = channel[Watcher](capacity=4)    #$ error[E7015]: `Watcher` holds `target`, a `sharedref mut Player` link, which other names on this thread may change, so no channel carries a `Watcher`; let a scoped task use `w` where it is (`scope.spawn(owns fn(): ...)`) instead of sending it
    println(rx.try_recv().is_none())
