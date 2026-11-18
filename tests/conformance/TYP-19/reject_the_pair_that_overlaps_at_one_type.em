#$ test: compile-fail
#$ rules: TYP-19
## `[TYP-19]` - two implementations overlap when one type could be both, each one's type parameters standing
## for any type: `Pair[T, int]` and `Pair[StrView, U]` overlap at `Pair[StrView, int]`, whether or not any
## instance meets both.

interface Mark:
    fn mark() -> int

[A, B]
struct Pair:
    a: A
    b: B

extend[T] Pair[T, int] implements Mark:
    fn mark() -> int:
        return 1

extend[U] Pair[StrView, U] implements Mark:   #$ error[E2041]: `Pair[StrView, U]` and `Pair[T, i64]` can be one type, and both implement `Mark`
    fn mark() -> int:
        return 2

fn main():
    println(1)
