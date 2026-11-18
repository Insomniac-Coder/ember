#$ test: run-pass
#$ rules: TYP-19
#$ stdout: 1 2
## `[TYP-19]` - a bound separates two implementations where the type it bounds is written out in full and does
## not meet it: `extend[T: needs Copy] Pair[T, int]` and `extend Pair[String, int]` do not overlap, a `String`
## is no `Copy`.

interface Mark:
    fn mark() -> int

[A, B]
struct Pair:
    a: A
    b: B

extend[T: needs Copy] Pair[T, int] implements Mark:
    fn mark() -> int:
        return 1

extend Pair[String, int] implements Mark:
    fn mark() -> int:
        return 2

fn main():
    println(Pair(3, 4).mark(), Pair(String("a"), 4).mark())
