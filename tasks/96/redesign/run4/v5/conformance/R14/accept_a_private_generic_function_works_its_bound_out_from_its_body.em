#$ test: run-pass
#$ rules: TYP-44, BLD-14
#$ stdout: 6
## design rules: R14, R17
## R14: "A non-`pub` function's bounds are worked out from its body" (AN-69's `sum`).

[T]
fn sum(xs: Span[T]) -> T:
    acc = copy of xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

fn main():
    xs = [1, 2, 3]
    println(sum(xs.as_span()))
