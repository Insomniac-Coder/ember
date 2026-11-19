#$ test: compile-fail
#$ rules: THR-9, THR-8, THR-1
# `[THR-9]` — a `thread_shared T` needs a `T` that is `Send` and `ThreadShared`: every owner's thread
# reads the same `T`.

fn main():
    _h = thread_shared Cell(1)    #$ error[E7015]: several threads may not read a `Cell[i64]` at once
