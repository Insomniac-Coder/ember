#$ test: run-pass
#$ rules: TYP-36, STR-5
#$ profiles: debug, development, release
#$ stdout: true false true false
#$ stdout: true true false
#$ stdout: true false
#$ stdout: true false
# `[TYP-36]`, IX.1 (ODR-241) — two boxes compare with `==` when what they hold does: `Box[T]` is
# `Eq` when `T` is, so it meets a `T: Eq` bound, an `Array` or an `Option` of boxes compares, and
# a recursive type through a `Box` derives `Eq` (the largest set whose fields are `Eq`, the type
# taken as `Eq`). A held type's written `eq` is what decides.

[T: needs Eq]
fn same(a: T, b: T) -> bool:
    return a == b

@derive(Eq)
struct Node:
    value: int
    next: Option[owns Node]

## Equal when the last digits are.
struct Digit:
    n: int

    fn eq(other: Digit) -> bool:
        return this.n % 10 == other.n % 10

fn main():
    three: owns int = owns 3
    also_three: owns int = owns 3
    four: owns int = owns 4
    println(three == also_three, three == four, same(three, also_three), same(three, four))
    xs: Array[owns int] = [owns 1, owns 2]
    ys: Array[owns int] = [owns 1, owns 2]
    o: Option[owns int] = Some(owns 5)
    five: Option[owns int] = Some(owns 5)
    six: Option[owns int] = Some(owns 6)
    println(xs == ys, o == five, o == six)
    one = Node(1, Some(owns Node(2, None)))
    also = Node(1, Some(owns Node(2, None)))
    other = Node(1, Some(owns Node(3, None)))
    println(one == also, one == other)
    twelve: owns Digit = owns Digit(12)
    two: owns Digit = owns Digit(2)
    three_digit: owns Digit = owns Digit(3)
    println(twelve == two, same(twelve, three_digit))
