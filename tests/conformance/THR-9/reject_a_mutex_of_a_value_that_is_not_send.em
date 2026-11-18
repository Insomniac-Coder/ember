#$ test: compile-fail
#$ rules: THR-9, THR-1
# `[THR-9]` — a `Mutex[T]` is `ThreadShared` only when `T` is `Send`: a `shares` owner never is.

from std.sync import Mutex

class Holder:
    held: Mutex[shares mut int]

    fn init(owns value: shares mut int):
        this.held = Mutex(value)

fn main():
    _h = thread_shared Holder(shares mut 1)    #$ error[E7015]: `Holder.held` is a `std.sync.Mutex[shares mut i64]`
