#$ test: compile-fail
#$ rules: CTL-10
## `[CTL-10]` - different types in the arms of a `match` statement are `E2230`, naming each arm's.

fn main():
    n = 1
    match n:
        0:
            v = 1    #$ error[E2230]: `v` is assigned in every branch, but at different types
        _:
            v = "many"
    println(v)
