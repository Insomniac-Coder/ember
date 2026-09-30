#$ test: run-pass
#$ rules: TYP-21, TYP-16, IFC-1, TYP-24
#$ stdout: 2 true 2 100
# D-400 — a generic extension implementing an instance of a generic
# interface (`Conv[Array[T]]`, `Add[Wrap[T]]`) gives its methods to that
# instance at each type it applies to, as a concrete block does (D-313), so
# two instances of one interface can share method names. The methods were
# the type's own: the second `conv` was `E1030` and each `E2040`.

interface Conv[T]:
    fn conv(self) -> T

struct Wrap[T]:
    x: T

extend[T: Copy] Wrap[T] implements Conv[Array[T]]:
    fn conv(self) -> Array[T]:
        return [self.x, self.x]

extend[U] Wrap[U] implements Conv[bool]:
    fn conv(self) -> bool:
        return true

extend[T: Copy] Wrap[T] implements Add[int]:
    type Output = int
    fn add(self, other: int) -> int:
        return other + 1

extend[T: Copy] Wrap[T] implements Add[Wrap[T]]:
    type Output = int
    fn add(self, other: Wrap[T]) -> int:
        return 100

fn as_list[C: Conv[Array[int]]](c: C) -> int:
    return c.conv().len()

fn as_flag[C: Conv[bool]](c: C) -> bool:
    return c.conv()

fn main():
    w = Wrap(x = 5)
    println(as_list(w), as_flag(w), w + 1, w + Wrap(x = 6))
