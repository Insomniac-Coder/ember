#$ test: compile-fail
#$ rules: CTL-0
#$ error[E2035]: condition must be `bool`
#$ error[E2035]: condition must be `bool`

## `[CTL-0]` — there is no truthiness conversion, and the fix depends on the
## type, so the diagnostic names the call that produces a `bool`.
fn main():
    n: i32 = 3
    if n:
        println(n)
    while n:
        n = n - 1
