#$ test: compile-fail
#$ rules: GRM-31
#$ error[E2260]: a `some` type is legal only in a function's return type
#$ error[E2260]: a `some` type is legal only in a function's return type
#$ error[E2260]: a `some` type is legal only in a function's return type
# `[GRM-31]` — `some I` names the one type a function's body returns, so it is legal only in a
# function's return type (or nested inside one): a parameter, a field or a local's annotation is
# `E2260`.

interface Shape:
    fn area(self) -> float

struct Holder:
    shape: some Shape

fn measure(s: some Shape) -> float:
    return s.area()

fn main():
    x: some Shape = 1
