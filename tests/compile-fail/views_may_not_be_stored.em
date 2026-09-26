#$ test: compile-fail
#$ rules: TYP-15
#$ error[E3063]: is a view, so it may not be stored in an `Array` unless it is `static`

## `[TYP-15]` — a view may live in a local, a parameter, a return value or an
## `Option`/`Result` payload, and in storage with no bounding region (a
## `static`, a class field, a `Box`, a container's element) only if it is
## `static` (ODR-069). A reference to a local is not.

fn main():
    n = 5
    xs: Array[ref i64] = Array[ref i64]()
    xs.push(ref n)
    println(xs.len())
