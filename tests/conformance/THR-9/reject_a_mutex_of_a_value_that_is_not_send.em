#$ test: compile-fail
#$ rules: THR-9, THR-1
#$ error[E7001]: which is not Send or Sync
# `[THR-9]` — a `Mutex[T]` is `Sync` only when `T` is `Send`: a `Shared` never is (`[HEAP-10]`).

from std.sync import Mutex

@sync
class Holder:
    held: Mutex[Shared[int]]

    fn init(mut self, owned value: Shared[int]):
        self.held = Mutex(value)

fn main():
    h = Holder(Shared(1))
