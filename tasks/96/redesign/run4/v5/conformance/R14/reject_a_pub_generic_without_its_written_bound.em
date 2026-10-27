#$ test: compile-fail
#$ rules: IFC-6, TYP-17, BLD-14
## design rules: R14, R17
## R14: "a `pub` one writes them" (AN-69's `total`, in section 3's form: one operation, so the one line
## the missing bound fails on carries the message); R17: across a package's edge only the written word
## counts. Section 3's E2040 message.

[T]
pub fn total(a: T, b: T) -> T:
    return a + b    #$ error[E2040]: `total` is `pub`, so its bound is written: write `[T: Add[T, Output = T]]` in place of `[T]` above `pub fn total(a: T, b: T) -> T:`

fn main():
    println(total(1, 2))
