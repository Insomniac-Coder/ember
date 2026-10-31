#$ test: compile-fail
#$ rules: LT-1a, ATT-1
## `[ATT-1]`, `[LT-1a]` — `@borrows` is gone: what a result points into is `from p` after its type.

@borrows(source = source)    #$ error[E0104]: `@borrows` is gone
fn first(source: Span[i32]) -> Span[i32]:
    return source

fn main():
    println(0)
