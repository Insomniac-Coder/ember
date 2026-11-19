#$ test: compile-fail
#$ rules: HEAP-10, THR-1
# `[HEAP-10]`, `[THR-1]` — a `shares` owner is never `Send` or `ThreadShared`: its counts are plain memory.

class Holder:
    held: shares mut int

fn main():
    _h = thread_shared Holder(shares mut 1)    #$ error[E7015]: `Holder.held` is a `shares mut i64` owner
