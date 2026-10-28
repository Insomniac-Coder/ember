#$ test: compile-fail
#$ rules: R14, R17
## R14: "a `pub` one writes them" (AN-69's `total`); R17: across a package's edge only the written word
## counts.

[T]
pub fn total(xs: Span[T]) -> T:
    acc = copy of xs[0]
    for x in xs[1..]:
        acc = acc + x    #$ error[E2040]: `total` is `pub`, so its bound is written: write `[T: Add[T, Output = T] + Copy]` above `pub fn total(xs: Span[T]) -> T:`
    return acc

fn main():
    xs = [1, 2, 3]
    println(total(xs.as_span()))
