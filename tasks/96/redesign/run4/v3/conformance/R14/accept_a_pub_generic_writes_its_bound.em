#$ test: run-pass
#$ rules: R14, R17
#$ stdout: 6
## R14: a `pub` generic function writes its bounds (AN-69's `total`, with the line E2040 gives).

[T: Add[T, Output = T] + Copy]
pub fn total(xs: Span[T]) -> T:
    acc = copy of xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

fn main():
    xs = [1, 2, 3]
    println(total(xs.as_span()))
