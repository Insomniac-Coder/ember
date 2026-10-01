#$ test: run-pass
#$ rules: TYP-21, IFC-1
#$ stdout: 1 true 3 1 3 1
# D-429 — D-400's two instances of one generic interface on a generic type
# hold for a generic class, and for two blocks whose interface arguments
# are concrete. A generic class's extension methods each took the block's
# first interface, so `Conv[Array[T]]` and `Conv[bool]` gave it one `conv`
# twice (`E1030`, `E2040`); and the generic body of `Add[int]`'s `add` was
# checked against `Add[bool]`'s signature (`+` on a `bool`), for structs
# and classes alike.

interface Conv[T]:
    fn conv(self) -> T

class Holder[T]:
    x: T

extend[T: Copy] Holder[T] implements Conv[Array[T]]:
    fn conv(self) -> Array[T]:
        return [self.x]

extend[U] Holder[U] implements Conv[bool]:
    fn conv(self) -> bool:
        return true

struct Pair2[T]:
    x: T

extend[T] Pair2[T] implements Add[int]:
    type Output = int
    fn add(self, other: int) -> int:
        return other + 1

extend[T] Pair2[T] implements Add[bool]:
    type Output = int
    fn add(self, other: bool) -> int:
        if other:
            return 1
        return 0

class Counter[T]:
    x: T

extend[T] Counter[T] implements Add[int]:
    type Output = int
    fn add(self, other: int) -> int:
        return other + 1

extend[T] Counter[T] implements Add[bool]:
    type Output = int
    fn add(self, other: bool) -> int:
        if other:
            return 1
        return 0

fn main():
    h = Holder(x = 5)
    a: Array[int] = Conv[Array[int]].conv(h)
    b: bool = Conv[bool].conv(h)
    p = Pair2(x = "s")
    c = Counter(x = "s")
    println(a.len(), b, p + 2, p + true, c + 2, c + true)
