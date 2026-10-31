#$ test: run-pass
#$ rules: TYP-19, TYP-20
#$ stdout: copy text 1 2 array option
# ODR-092 — implementations whose types can never be one type are each kept:
# a bound separates two where the type it bounds is written out in full and
# does not meet it (`String` is not `Copy`), and different instances,
# different shapes and different interface arguments never meet.

interface Named:
    fn name() -> StrView

interface Kind:
    fn kind() -> StrView

[A, B]
struct Pair:
    a: A
    b: B

[T]
struct Wrap:
    x: T

extend[T: needs Copy] Pair[T, int] implements Named:
    fn name() -> StrView:
        return "copy"

extend[U] Pair[String, U] implements Named:
    fn name() -> StrView:
        return "text"

struct V:
    x: int

extend V implements Add[int]:
    type Output = int
    fn add(other: int) -> int:
        return this.x + other

extend V implements Add[V]:
    type Output = int
    fn add(other: V) -> int:
        return this.x + other.x + 1

extend[T] Wrap[Array[T]] implements Kind:
    fn kind() -> StrView:
        return "array"

extend[T] Wrap[Option[T]] implements Kind:
    fn kind() -> StrView:
        return "option"

fn main():
    p = Pair(a = 5, b = 1)
    q = Pair(a = String.from("s"), b = 1)
    println(p.name(), q.name(), V(x = 0) + 1, V(x = 0) + V(x = 1), Wrap(x = [1]).kind(), Wrap(x = Some(1)).kind())
