#$ test: compile-fail
#$ rules: TYP-20, TYP-19
# `[TYP-20]` — "Two implementations of one interface for one type anywhere in
# a program are `E2041`, naming both." The error named only the second.

interface Named:
    fn name(self) -> str

struct P:
    x: int

extend P implements Named:
    fn name(self) -> str:
        return "one"

extend P implements Named:                     #$ error[E2041]: the first implementation is here
    fn name(self) -> str:
        return "two"

fn main():
    println(P(x = 1).name())
