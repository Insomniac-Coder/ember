#$ test: compile-fail
#$ rules: TYP-19
## `[TYP-19]` - a bound on a type left open never separates two implementations: a type meeting both bounds can
## be declared later, in another package.

interface Mark:
    fn mark() -> int

[A, B]
struct Pair:
    a: A
    b: B

extend[T: needs Copy] Pair[T, StrView] implements Mark:
    fn mark() -> int:
        return 3

extend[U: needs Display] Pair[U, StrView] implements Mark:   #$ error[E2041]: `Pair[U, StrView]` and `Pair[T, StrView]` can be one type, and both implement `Mark`
    fn mark() -> int:
        return 4

fn main():
    println(1)
