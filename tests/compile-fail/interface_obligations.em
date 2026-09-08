#$ test: compile-fail
#$ rules: IFC-3, TYP-19, TYP-24
#$ error[E2040]: `Missing` implements `Named` but does not define `label`
#$ error[E2041]: `Twice` already implements `Named`
#$ error[E2070]: `label` is offered by both
#$ error[E2140]: a `mut` argument must be a variable, not a value

interface Named:
    fn label(self) -> i32

interface Tagged:
    fn label(self) -> i32

struct Missing implements Named:
    x: i32

struct Twice implements Named, Named:
    x: i32
    fn label(self) -> i32:
        return 0

struct Both:
    x: i32

extend Both implements Named:
    fn label(self) -> i32:
        return 1

extend Both implements Tagged:
    fn label(self) -> i32:
        return 2

fn double(mut n: i32):
    n = n + n

fn main():
    b = Both(1)
    println(b.label())
    double(1 + 1)
