# Clause IFC-6.2: "A `pub` function writes them (`[MOD-9]`): a missing one is `E2040`, one message,
# on the first line of the body that needs a bound the function does not write, giving every bound
# the body needs"
#$ test: compile-fail
#$ rules: IFC-6, TYP-17, MOD-9, EFF-4
# The spec's message.

[T]
pub fn total(a: T, b: T) -> T:
    return a + b    #$ error[E2040]: `total` is `pub`, so its bound is written: write `[T: needs Add[T, Output = T]]` in place of `[T]` above `pub fn total(a: T, b: T) -> T:`

fn main():
    println(total(1, 2))
