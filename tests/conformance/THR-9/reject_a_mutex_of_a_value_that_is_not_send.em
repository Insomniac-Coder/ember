#$ test: compile-fail
#$ rules: THR-9, THR-1
#$ error[E7001]: which is not Send or Sync
# `[THR-9]` — a `Mutex[T]` is `Sync` only when `T` is `Send`: a `Shared` never is (`[HEAP-10]`).

from std.sync import Mutex

@sync
class Holder:
    held: Mutex[shares mut int]

    fn init(owns value: shares mut int):
        this.held = Mutex(value)

fn main():
    h = Holder(shares mut 1)
