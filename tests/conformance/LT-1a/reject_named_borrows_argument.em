#$ test: compile-fail
#$ rules: LT-1a

@borrows(source = source)
fn first(source: Span[i32]) -> Span[i32]: #$ error[E2031]: each `@borrows` argument must be a parameter name
    return source

fn main():
    println(0)
