#$ test: compile-fail
#$ rules: R4
#$ help: fn take(n: int) -> int:
## R4: `owns x: T` takes the value over, and is refused where `T` is known to be `Copy`: an `int` is
## copied, not handed over (AN-43).

fn take(owns n: int) -> int:    #$ error[E2085]: an `int` is copied, not handed over; remove `owns`: write `fn take(n: int) -> int:`
    return n + 1

fn main():
    println(take(1))
