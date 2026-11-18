#$ test: compile-fail
#$ rules: THR-1, THR-8, THR-9
#$ profiles: debug
# `[THR-1]` — a generic class is judged for the instance made `thread_shared`, to the field at fault.

class Local:
    value: Cell[int] = Cell(1)

[T]
class Holder:
    value: T

    fn init(owns value: T):
        this.value = value

fn main():
    _plain = Holder(1)
    _holder = thread_shared Holder(Local())    #$ error[E7015]: `Local.value` is a `Cell[i64]`
