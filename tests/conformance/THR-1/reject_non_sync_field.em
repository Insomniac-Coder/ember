#$ test: compile-fail
#$ rules: THR-1, THR-8, THR-9, CLS-8
#$ profiles: debug
# `[THR-1]` — a part that is not `ThreadShared`, reachable from a `thread_shared` value, is refused
# where the value is made, naming the field.

class Local:
    value: Cell[int] = Cell(0)

class Shared:
    item: Local

    fn init(owns item: Local):
        this.item = item

fn main():
    _s = thread_shared Shared(Local())    #$ error[E7015]: `Local.value` is a `Cell[i64]`
