#$ test: run-pass
#$ rules: BLD-14, TYP-44
#$ stdout: 5 5
## design rules: R17, R14
## R17: inside a package a fact is used freely: a function that is not `pub` gets its bounds from its
## body; a `pub` one writes them (AN-69).

[T]
fn add_two(a: T, b: T) -> T:
    return a + b

[T: Add[T, Output = T]]
pub fn total(a: T, b: T) -> T:
    return a + b

fn main():
    println(add_two(2, 3), total(2, 3))
