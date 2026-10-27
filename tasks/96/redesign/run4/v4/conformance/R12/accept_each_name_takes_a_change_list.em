#$ test: run-pass
#$ rules: R12, R4
#$ stdout: 3
#$ stdout: 1
## Verdicts 21 and 50, R12 (rules.md lines 659-665): `C` is filled at each call from the function given:
## the first call changes `main`'s own `names` (no clause in `main`), the second changes nothing. AN-56's
## NEW program, with verdict 71's `fn(owns s)`: a lambda's parameter says what it receives, matching
## the function type (`fn(owns String)`); `fn(s)` there is E2020 (reject_a_plain_lambda_parameter_where_each_name_hands_the_name_over).

[C]
fn each_name(owns f: fn(owns String) changes C) changes C:
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owns s) changes names: names.push(s))
    each_name(fn(owns s) => println(s.len()))
    println(names.len())
