#$ test: compile-fail
#$ rules: HEAP-10, THR-1
#$ error[E7001]: field `Holder.held` has type `Shared[i64]`, which is not Send or Sync
# `[HEAP-10]` — a `Shared` is never `Send` or `Sync`: its counts are plain memory.

@sync
class Holder:
    held: Shared[int]

fn main():
    h = Holder(Shared(1))
