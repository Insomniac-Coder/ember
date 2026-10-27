#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 2
## Verdict 75, R12 (rules.md lines 608-611): the lambda names `main`'s real `hits` by its name; `main`
## wrote the lambda, so the change is of its own variable and its first line names nothing. The nearest
## allowed program to reject_a_lambda_that_changes_a_captured_atomic_without_naming_it.

from std.sync import Atomic

fn main():
    hits = Atomic(0)
    count_hit = fn() changes hits: hits.fetch_add(1)
    count_hit()
    count_hit()
    println(hits.load())
