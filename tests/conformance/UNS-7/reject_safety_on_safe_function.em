#$ test: compile-fail
#$ rules: UNS-7, ATT-1
#$ profiles: debug
#$ error[E0104]: `@safety` applies only to an `unsafe fn`

@safety("No obligation")
fn operation(x: i32) -> i32:
    return x
