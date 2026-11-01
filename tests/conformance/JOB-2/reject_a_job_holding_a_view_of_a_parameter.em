#$ test: compile-fail
#$ rules: JOB-2, LT-42
#$ error[E3063]: `owned fn` captures `StrView` by value, but that view is not static
# `[JOB-2]` — a submitted job may outlive every local of the code that submitted it, so it holds
# no view but a static one: a parameter's text is refused where the `owned fn` is made.

import std.jobs

fn show(text: StrView):
    jobs.submit(owns owned fn() => println(text))

fn main():
    show("x")
