#$ test: run-pass
#$ rules: TYP-24, TYP-21
#$ stdout: 5 0 6 -1
# D-440 — two instances of one generic interface may each give an associated
# function of one name, generic or not; the arguments choose, as for a method
# (ODR-096). The second used to replace the first, whose body was then checked
# against the other's signature, and the C had one function twice.

interface Make[T]:
    fn make(x: T) -> Self
    fn make_with[U](x: T, y: U) -> Self

struct S:
    n: int

extend S implements Make[int]:
    fn make(x: int) -> S:
        return S(n = x)

    fn make_with[U](x: int, y: U) -> S:
        return S(n = x + 1)

extend S implements Make[bool]:
    fn make(x: bool) -> S:
        return S(n = 0)

    fn make_with[U](x: bool, y: U) -> S:
        return S(n = -1)

fn main():
    println(S.make(5).n, S.make(true).n, S.make_with(5, "a").n, S.make_with(false, 2).n)
