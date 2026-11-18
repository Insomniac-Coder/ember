#$ test: compile-fail
#$ rules: LEX-18
## `[LEX-18]` - `1.` before an identifier is a member access on `1`, not the start of a float: `1.e3` is
## the field `e3` of an integer.

fn main():
    x = 1.e3        #$ error[E2020]: has no field `e3`
    println(x)
