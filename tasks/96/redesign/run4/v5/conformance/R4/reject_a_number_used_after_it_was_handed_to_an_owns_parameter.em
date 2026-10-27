#$ test: compile-fail
#$ rules: R4, R1
#$ help: to keep your own, write `m = take(copy of n)` in place of line 14
## R4, R1: `owns x: T` takes the value over for every type, numbers included (verdict 84): the caller's
## name ends at the call, as for any other type, and a later use is refused (E3040). The machine copies
## the number either way. The nearest program that is allowed is
## `accept_a_number_given_to_an_owns_parameter_or_a_copy_of_one`.

fn take(owns x: int) -> int:
    return x + 1

fn main():
    n = 1
    m = take(n)
    println(n, m)              #$ error[E3040]: `n` was handed over to `take` on line 14 and is used on line 15
