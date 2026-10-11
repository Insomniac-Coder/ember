#$ test: compile-fail
#$ rules: THR-1, THR-8, THR-9, CLO-3

# ODR-102 — an owned callable value hides what it captured, so it is neither
# `Send` nor `Sync`, and a `@sync` class cannot hold one.

@sync
class Job:
    run: fn() -> int    #$ error[E7001]: field `Job.run` has type `fn() -> i64`, which is not Send or Sync

fn main():
    pass
