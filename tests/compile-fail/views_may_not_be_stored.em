#$ test: compile-fail
#$ rules: TYP-15
#$ error[E3063]: is a view, so it may not be stored in a container element

## `[TYP-15]` — a view may live in a local, a parameter, a return value or an
## `Option`/`Result` payload, and nowhere with no bounding region: a `static`,
## a class or non-view struct field, a `Box`, a container element.
##
## `[TYP-15a]`'s `BorrowList[T]` and `ViewList[T]` are the sanctioned exception
## and are not built; arbitrary owning containers stay rejected.

fn main():
    xs: Array[ref i32] = Array[ref i32]()
    println(xs.len())
