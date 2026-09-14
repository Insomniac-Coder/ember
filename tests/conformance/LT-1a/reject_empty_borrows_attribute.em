#$ test: compile-fail
#$ rules: LT-1a

@borrows()
fn first(values: Span[i32]) -> Span[i32]: #$ error[E2031]: `@borrows` must name at least one parameter
    return values

fn main():
    println(0)
