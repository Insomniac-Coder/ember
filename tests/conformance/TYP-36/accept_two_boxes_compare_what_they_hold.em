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

fn same[T: Eq](a: T, b: T) -> bool:
    return a == b

@derive(Eq)
struct Node:
    value: int
    next: Option[Box[Node]]

## Equal when the last digits are.
struct Digit:
    n: int

    fn eq(self, other: Digit) -> bool:
        return self.n % 10 == other.n % 10

fn main():
    println(Box(3) == Box(3), Box(3) == Box(4), same(Box(3), Box(3)), same(Box(3), Box(4)))
    xs = [Box(1), Box(2)]
    ys = [Box(1), Box(2)]
    o: Option[Box[int]] = Some(Box(5))
    println(xs == ys, o == Some(Box(5)), o == Some(Box(6)))
    one = Node(1, Some(Box(Node(2, None))))
    also = Node(1, Some(Box(Node(2, None))))
    other = Node(1, Some(Box(Node(3, None))))
    println(one == also, one == other)
    println(Box(Digit(12)) == Box(Digit(2)), same(Box(Digit(12)), Box(Digit(3))))
