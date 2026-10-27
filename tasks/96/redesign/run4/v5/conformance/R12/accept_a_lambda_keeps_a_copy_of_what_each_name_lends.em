#$ test: run-pass
#$ rules: R12, R4
#$ stdout: 1
## Verdict 71, R12 (rules.md lines 676-680): the lambda's plain parameter matches `fn(String)`, which
## lends each name, and it keeps a copy of its own in `main`'s `names` (no word on any first line,
## verdict 86). The nearest allowed program to reject_a_lambda_that_takes_over_what_each_name_lends
## (completeness G5).

fn each_name(owns f: fn(String)):
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(s): names.push(copy of s))
    println(names.len())
