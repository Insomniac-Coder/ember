#$ test: compile-fail
#$ rules: LT-1a

@borrows(0)
fn first(values: Span[i32]) -> Span[i32]: #$ error[E2031]: each `@borrows` argument must be a parameter name
    return values

fn main():
    println(0)
