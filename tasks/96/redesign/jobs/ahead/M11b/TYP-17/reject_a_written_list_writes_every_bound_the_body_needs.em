# Clause TYP-17.2: "a list that leaves out one the body needs is `E2040`, whose message gives the
# whole line" (`[IFC-6]`: "A function that is not `pub` and writes some of them writes every one
# its body needs")
#$ test: compile-fail
#$ rules: TYP-17, IFC-6
# The spec's message, with this file's line number; `Lookalike` declares an ordinary `fn add` and
# is not `Add`.

interface Lookalike:
    fn add(other: int) -> int

[T: needs Lookalike]
fn direct_add(a: T, b: T) -> T:
    return a + b    #$ error[E2040]: `direct_add` writes what `T` needs, so it writes all of it: `a + b` (line 14) needs `Add`, which `Lookalike` does not give; write `[T: needs (Lookalike, Add[T, Output = T])]` in place of `[T: needs Lookalike]`

fn main():
    pass
