#$ test: compile-fail
#$ rules: TYP-5
#$ help: bind the value to a local first
# `[TYP-5]` rule 7 borrows a place: a temporary has nothing a `ref` could
# point into after the statement.

struct Counter:
    n: int

fn peek(c: ref Counter) -> int:
    return c.n

fn main():
    println(peek(Counter(n=7)))    #$ error[E2020]: expected `ref Counter`, found `Counter`
