#$ test: compile-fail
#$ rules: LT-1a, LT-4b

@borrows(count)    #$ error[E2031]: `@borrows` names `count`, which a result cannot borrow
fn first(count: usize, values: Span[i32]) -> Span[i32]:
    return values

fn main():
    println(0)
