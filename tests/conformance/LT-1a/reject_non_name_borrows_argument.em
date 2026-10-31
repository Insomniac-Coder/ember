#$ test: compile-fail
#$ rules: LT-1a, ATT-1
## `[ATT-1]`, `[LT-1a]` — `@borrows` is gone: what a result points into is `from p` after its type.

@borrows(0)    #$ error[E0104]: `@borrows` is gone
fn first(values: Span[i32]) -> Span[i32]:
    return values

fn main():
    println(0)
