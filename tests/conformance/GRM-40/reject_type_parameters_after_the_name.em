#$ test: compile-fail
#$ rules: GRM-40
## `[GRM-40]` — brackets after a function's, struct's or class's name are `E0100`, the help the two
## lines to write.

fn first[T](xs: Array[T]) -> T:    #$ error[E0100]: type parameters go on a line of their own, above the declaration
    return xs[0]

struct Holder[T]:    #$ error[E0100]: type parameters go on a line of their own, above the declaration
    value: T

fn main():
    println(first([1]))
