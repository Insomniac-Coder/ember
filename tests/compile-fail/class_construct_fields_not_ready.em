#$ test: compile-fail
#$ rules: CLS-1, CLS-2
#$ error[E1010]: class construction for `NeedsInit` is not implemented yet in this phase

class NeedsInit:
    value: i32

fn main():
    value = NeedsInit()
    println(1)
