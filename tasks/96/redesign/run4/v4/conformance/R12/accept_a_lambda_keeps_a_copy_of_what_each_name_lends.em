#$ test: run-pass
#$ rules: R12, R4
#$ stdout: 1
## Verdict 71, R12 (rules.md lines 627-630): the lambda's plain parameter matches `fn(String)`, which lends
## each name, and it keeps a copy of its own; `C` is `main`'s own `names` (no clause in `main`). The
## nearest allowed program to reject_a_lambda_that_takes_over_what_each_name_lends (completeness G5).

[C]
fn each_name(owns f: fn(String) changes C) changes C:
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(s) changes names: names.push(copy of s))
    println(names.len())
