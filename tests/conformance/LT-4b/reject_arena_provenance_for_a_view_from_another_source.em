#$ test: compile-fail
#$ rules: LT-1a, LT-4a, LT-4b

@borrows(arena)
fn unrelated(arena: Arena, values: Span[i32]) -> Span[i32]:
    return values #$ error[E3062]: the returned view points into `values`, which `@borrows` does not name

fn main():
    println(0)
