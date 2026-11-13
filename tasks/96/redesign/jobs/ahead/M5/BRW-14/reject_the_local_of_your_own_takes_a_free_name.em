# Clause BRW-14.3: "The local of your own such a message gives is named `new_` and the parameter's
# name, or, where the function already has that name, the first of `new_n_2`, `new_n_3`, ... it does
# not have."
#$ test: compile-fail
#$ rules: BRW-14, DIA-25
# Only the name is pinned (the rest of each message is BRW-14.1's and BRW-14.4's).

fn taken(n: int) -> int:
    new_n = n + 1
    n = new_n * 2    #$ error[E3023]: write `new_n_2 = n`
    return n

fn taken_twice(n: int) -> int:
    new_n = 1
    new_n_2 = 2
    n = new_n + new_n_2    #$ error[E3023]: write `new_n_3 = new_n + new_n_2` in place of line
    return n

fn main():
    println(taken(1), taken_twice(1))
