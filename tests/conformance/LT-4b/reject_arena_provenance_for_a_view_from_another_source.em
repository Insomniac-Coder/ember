#$ test: compile-fail
#$ rules: LT-1a, LT-4a, LT-4b

fn unrelated(arena: Arena, values: Span[i32]) -> Span[i32] from arena:
    return values #$ error[E3062]: the returned view points into `values`, but its first line says `from arena`

fn main():
    println(0)
