#$ test: compile-fail
#$ rules: TYP-36, TYP-17
## `[TYP-36]` — an `Array[T]` or a `[T; N]` is `Ord` only when `T` is: a struct without
## `@derive(Ord)` is not, so neither is a list of them (D-854).

struct Point:
    x: int

fn bigger[T: Ord](a: T, b: T) -> bool:
    return a > b

fn main():
    a = [Point(1)]
    b = [Point(2)]
    println(a < b)  #$ error[E2040]: `Array[Point]` does not implement `Ord`
    lists = [a, b]
    lists.sort()  #$ error[E2040]: `Array[Point]` does not implement `Ord`
    pair: [Point; 1] = [Point(3)]
    println(bigger(pair, pair))  #$ error[E2040]: `[Point; 1]`
