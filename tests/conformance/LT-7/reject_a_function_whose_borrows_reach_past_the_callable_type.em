#$ test: compile-fail
#$ rules: LT-7, LT-1a
#$ help: call `slot` directly
# A callable type carries no `@borrows`: its result borrows only its source
# parameters. `slot` borrows a `mut int`, so it is not a value of that type.

@borrows(n)
fn slot(mut n: int) -> ref int:
    return ref n

fn main():
    g: fn(mut int) -> ref int = slot    #$ error[E2020]: `slot` is not a value of type `fn(mut i64) -> ref i64`
    x = 5
    println(g(x))
