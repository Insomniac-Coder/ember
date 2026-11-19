#$ test: compile-fail
#$ rules: THR-1, THR-8, THR-9, CLO-3
# ODR-102 — an owned callable value hides what it captured, so it is neither `Send` nor
# `ThreadShared`, and a `thread_shared` object cannot hold one.

class Job:
    run: fn() -> int

fn main():
    _j = thread_shared Job(owned fn() => 1)    #$ error[E7015]: `Job.run` is a `fn() -> i64`
