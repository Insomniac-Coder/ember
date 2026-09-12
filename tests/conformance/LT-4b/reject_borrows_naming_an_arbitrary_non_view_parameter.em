#$ test: compile-fail
#$ rules: LT-1a, LT-4b

@borrows(count)
fn first(count: usize, values: Span[i32]) -> Span[i32]: #$ error[E2031]: `@borrows` names `count`, which is not view-typed
    return values

fn main():
    println(0)
