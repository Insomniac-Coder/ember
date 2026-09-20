#$ test: compile-pass
#$ rules: LT-1b
#$ not-help: write `@borrows(a, b)`

@borrows(a, b)
fn pick(a: Span[i32], b: Span[i32]) -> Span[i32]:
    return a
