#$ test: compile-fail
#$ rules: TYP-14, TYP-15
#$ error[E2030]: `Cursor` carries a borrow, so it is a view type

## `[TYP-14]` — a struct containing a `ref`, a `Span`, a `str` or another view
## type *is* a view type whatever it says. The attribute is required as
## documentation, because otherwise a reader has to check every field's type to
## know that the struct may not be stored in a field, a `static` or a container.

struct Cursor:
    at: ref i32
    step: i32

fn main():
    println(1)
