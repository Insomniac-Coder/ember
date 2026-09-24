#$ test: compile-fail
#$ rules: LT-1a

@borrows(source = source)    #$ error[E2031]: each `@borrows` argument must be a parameter name
fn first(source: Span[i32]) -> Span[i32]:
    return source

fn main():
    println(0)
