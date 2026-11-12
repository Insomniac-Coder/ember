#$ test: compile-fail
#$ rules: HEAP-10, THR-1
#$ error[E7001]: field `Holder.held` has type `shares mut i64`, which is not Send or Sync
# `[HEAP-10]` — a `shares` owner is never `Send` or `Sync`: its counts are plain memory. `[IDE-3]` — `main`,
# which no error is in, is checked on.

@sync
class Holder:
    held: shares mut int

fn main():
    h = Holder(shares mut 1)  #$ warning[L1001]: `h` is never read
