#$ test: compile-fail
#$ rules: TYP-34
#$ error[E2030]: `@view` on `Mode`, which holds no view; remove `@view`
## `[TYP-34]` - `@view` on a type that is not a view is `E2030`, an enum too.

@view
enum Mode:
    Fast
    Slow(int)

fn main():
    println(1)
