#$ test: compile-fail
#$ rules: GRM-40
## `[GRM-40]` — a type-parameter line goes on the line right above its function, struct or class: a
## blank line or a comment line between them is `E0100` (verdict 154). The parameters are still the
## declaration's, so nothing else is reported.

[T]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to

fn first(xs: Array[T]) -> T:
    return xs[0]

[T]    #$ error[E0100]: a `[...]` line gives type parameters, and goes on the line right above the function, struct or class they belong to
# a comment
struct Holder:
    value: T

fn main():
    println(first([1]), Holder[int](value=2).value)
