#$ test: compile-fail
#$ rules: JOB-2, THR-10
#$ error[E3063]: the job's function captures `text`, a view (`str`)
#$ error[E7004]: the job's function captures `shared`, a `Shared[i64]`, which is not Send
#$ error[E3014]: `jobs.scope()` must be bound by `with`
#$ error[E2020]: `jobs.submit` takes a function of no arguments that gives nothing, found `fn() -> i64`
# `[JOB-2]` — a submitted job may outlive every local of the code that submitted it, so it holds no
# view and what it captures is `Send`; a scope lives exactly as long as its `with` block; a job's
# function takes nothing and gives nothing.

import std.jobs

fn count() -> int:
    return 3

fn show(text: str):
    jobs.submit(owned fn() => println(text))

fn main():
    shared = Shared(0)
    jobs.submit(owned fn() => shared.get())
    loose = jobs.scope()
    jobs.submit(count)
    show("x")
