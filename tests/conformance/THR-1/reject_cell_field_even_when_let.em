#$ test: compile-fail
#$ rules: THR-1, THR-9
#$ profiles: debug
# `[THR-1]` — a `Cell` is no `ThreadShared` part, though no name changes it after `init`.

class Shared:
    let cell: Cell[int]

    fn init():
        this.cell = Cell(1)

fn main():
    _s = thread_shared Shared()    #$ error[E7015]: `Shared.cell` is a `Cell[i64]`, which several threads may not read at once
