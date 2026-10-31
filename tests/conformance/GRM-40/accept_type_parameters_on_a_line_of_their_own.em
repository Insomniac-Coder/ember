#$ test: run-pass
#$ rules: GRM-40, TYP-17
#$ stdout: 3 ab
#$ stdout: 7
## `[GRM-40]` — a function's, struct's or class's type parameters go on a line of their own, directly
## above it and below its attributes; what a type parameter needs is written after `needs`
## (`[TYP-17]`). The function and the type are used as before: `first(xs)`, `Holder[int]`.

[T: needs Copy]
fn first(xs: Array[T]) -> T:
    return xs[0]

@derive(Clone)
[T]
struct Holder:
    value: T

    [U: needs (Copy, Eq)]
    fn pick(other: U) -> U:
        return other

fn main():
    println(first([3, 4]), first(["ab"]))
    h = Holder[int](value=7)
    println(h.pick(h.value))
