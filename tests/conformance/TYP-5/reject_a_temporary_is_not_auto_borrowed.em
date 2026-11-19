#$ test: compile-fail
#$ rules: TYP-5
#$ help: bind the value to a local first
# `[TYP-5]` rule 7 borrows a place: a temporary has nothing a `ref` could
# point into after the statement.

struct Counter:
    n: int

fn main():
    r: ref Counter = Counter(n=7)    #$ error[E2020]: expected `ref Counter`, found `Counter`
    println(r.n)
