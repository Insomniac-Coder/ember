#$ test: compile-fail
#$ rules: LT-1a

@borrows()    #$ error[E2031]: `@borrows` must name at least one parameter
fn first(values: Span[i32]) -> Span[i32]:
    return values

fn main():
    println(0)
