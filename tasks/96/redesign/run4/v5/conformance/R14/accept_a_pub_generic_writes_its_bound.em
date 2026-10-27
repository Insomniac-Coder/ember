#$ test: run-pass
#$ rules: R14, R17
#$ stdout: 3
## R14: a `pub` generic function writes its bounds (the line E2040 gives).

[T: Add[T, Output = T]]
pub fn total(a: T, b: T) -> T:
    return a + b

fn main():
    println(total(1, 2))
