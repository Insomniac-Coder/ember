#$ test: compile-fail
#$ rules: TYP-17
## `[TYP-17]` — what a type parameter needs is written after `needs`, several in round brackets; 0.9.10's
## `T: A + B` is `E0100`, whose message gives the line with `needs`.

[T: Copy + Eq]    #$ error[E0100]: what a type needs is written after `needs`: write `[T: needs (Copy, Eq)]`
fn same(a: T, b: T) -> bool:
    return a == b

fn main():
    println(same(1, 1))
